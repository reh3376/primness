// h2-gpu: the GPU side of H2 (docs/h2_falsifiable_prediction.md).
//
//   h2-gpu DIR [REPEATS]
//
// Reads DIR/primes.bin and DIR/lanes.bin written by `h2-cpu --prepare` and
// times on the Metal device, after one warm-up run, REPEATS runs (default 5):
//   sieve   the same line of 2^29 positions and the same primes and roots as
//           h2-cpu. fill: work items of at most 256 updates of one prime,
//           appended to buckets of 2^14 positions through atomic counters in
//           device memory. apply: one threadgroup per bucket adds the updates
//           into two 16-bit counters per 32-bit word of threadgroup memory.
//           The checksum is the sum over positions of the count mod 256, as on
//           the CPU.
//   mulmod  x <- x * y mod n in Montgomery form with three 32-bit limbs
//           (R = 2^96), K = 2^16 steps, one lane per thread over all lanes;
//           the results x0 * y^K mod n of the CPU's lanes go to
//           DIR/gpu_lanes.bin.
//   copy    a 1 GiB copy, for the memory bandwidth reached.
// CSV lines on stdout: kernel,device,run,seconds,ops,ops_per_s,check, timed
// with the command buffers' GPU start and end times.
// Build: swiftc -O tools/h2_gpu/main.swift -o target/release/h2-gpu

import Foundation
import Metal

var standardError = FileHandleOutputStream(FileHandle.standardError)
struct FileHandleOutputStream: TextOutputStream {
    let handle: FileHandle
    init(_ h: FileHandle) { handle = h }
    mutating func write(_ s: String) { handle.write(s.data(using: .utf8)!) }
}

let args = CommandLine.arguments
guard args.count >= 2 else {
    FileHandle.standardError.write("Usage: h2-gpu DIR [REPEATS]\n".data(using: .utf8)!)
    exit(2)
}
let dir = args[1]
let repeats = args.count >= 3 ? Int(args[2]) ?? 5 : 5
let lineBits: UInt32 = 29
let regionBits: UInt32 = 14
let regions = 1 << Int(lineBits - regionBits)
let cap: UInt32 = 11_000
let itemUpdates: UInt64 = 256
let K: UInt32 = 1 << 16
let cpuLanes = 18 * 1024

let source = """
#include <metal_stdlib>
using namespace metal;

constant uint LINE_BITS = \(lineBits);
constant uint REGION_BITS = \(regionBits);
constant uint ITEM = \(itemUpdates);

kernel void fill(device const uint2* items [[buffer(0)]],
                 device atomic_uint* counts [[buffer(1)]],
                 device uint* upd [[buffer(2)]],
                 constant uint& cap [[buffer(3)]],
                 constant uint& nitems [[buffer(4)]],
                 uint i [[thread_position_in_grid]]) {
    if (i >= nitems) return;
    uint p = items[i].x;
    uint x = items[i].y;
    uint logp = 31 - clz(p);
    const uint line = 1u << LINE_BITS;
    const uint mask = (1u << REGION_BITS) - 1;
    for (uint k = 0; k < ITEM && x < line; k++) {
        uint b = x >> REGION_BITS;
        uint slot = atomic_fetch_add_explicit(&counts[b], 1, memory_order_relaxed);
        if (slot < cap) upd[b * cap + slot] = (x & mask) | (logp << 16);
        x += p;
    }
}

kernel void apply(device const uint* upd [[buffer(0)]],
                  device const uint* counts [[buffer(1)]],
                  device atomic_uint* part [[buffer(2)]],
                  constant uint& cap [[buffer(3)]],
                  threadgroup atomic_uint* tg [[threadgroup(0)]],
                  uint b [[threadgroup_position_in_grid]],
                  uint t [[thread_position_in_threadgroup]],
                  uint nt [[threads_per_threadgroup]],
                  uint lane [[thread_index_in_simdgroup]]) {
    const uint words = 1u << (REGION_BITS - 1);
    for (uint w = t; w < words; w += nt) atomic_store_explicit(&tg[w], 0u, memory_order_relaxed);
    threadgroup_barrier(mem_flags::mem_threadgroup);
    uint n = min(counts[b], cap);
    device const uint* u = upd + (ulong)b * cap;
    const uint mask = (1u << REGION_BITS) - 1;
    for (uint k = t; k < n; k += nt) {
        uint v = u[k];
        uint off = v & mask;
        atomic_fetch_add_explicit(&tg[off >> 1], (v >> 16) << ((off & 1) * 16), memory_order_relaxed);
    }
    threadgroup_barrier(mem_flags::mem_threadgroup);
    uint s = 0;
    for (uint w = t; w < words; w += nt) {
        uint c = atomic_load_explicit(&tg[w], memory_order_relaxed);
        s += (c & 0xffu) + ((c >> 16) & 0xffu);
    }
    s = simd_sum(s);
    if (lane == 0) atomic_fetch_add_explicit(&part[b], s, memory_order_relaxed);
}

struct Lane { uint4 n; uint4 x0; uint4 y; uint np32; uint pad0; ulong np64; uint4 r96sq; uint4 r128sq; uint4 pad1; };

inline uint3 mont(uint3 a, uint3 b, uint3 n, uint np) {
    uint t0 = 0, t1 = 0, t2 = 0, t3 = 0;
    for (int i = 0; i < 3; i++) {
        uint bi = b[i];
        ulong p = (ulong)a.x * bi + t0;            t0 = (uint)p;
        p = (ulong)a.y * bi + t1 + (p >> 32);      t1 = (uint)p;
        p = (ulong)a.z * bi + t2 + (p >> 32);      t2 = (uint)p;
        ulong s = (ulong)t3 + (p >> 32);           t3 = (uint)s;
        uint t4 = (uint)(s >> 32);
        uint m = t0 * np;
        p = (ulong)m * n.x + t0;
        p = (ulong)m * n.y + t1 + (p >> 32);       t0 = (uint)p;
        p = (ulong)m * n.z + t2 + (p >> 32);       t1 = (uint)p;
        s = (ulong)t3 + (p >> 32);                 t2 = (uint)s;
        t3 = t4 + (uint)(s >> 32);
    }
    bool ge = t3 != 0 || t2 > n.z || (t2 == n.z && (t1 > n.y || (t1 == n.y && t0 >= n.x)));
    if (ge) {
        uint b0 = t0 < n.x ? 1u : 0u;
        uint r0 = t0 - n.x;
        uint b1 = (t1 < n.y || (t1 == n.y && b0 == 1u)) ? 1u : 0u;
        uint r1 = t1 - n.y - b0;
        uint r2 = t2 - n.z - b1;
        return uint3(r0, r1, r2);
    }
    return uint3(t0, t1, t2);
}

kernel void mulmod(device const Lane* lanes [[buffer(0)]],
                   device uint4* out [[buffer(1)]],
                   constant uint& k [[buffer(2)]],
                   uint i [[thread_position_in_grid]]) {
    Lane l = lanes[i];
    uint3 n = l.n.xyz;
    uint3 x = mont(l.x0.xyz, l.r96sq.xyz, n, l.np32);
    uint3 y = mont(l.y.xyz, l.r96sq.xyz, n, l.np32);
    for (uint j = 0; j < k; j++) x = mont(x, y, n, l.np32);
    x = mont(x, uint3(1, 0, 0), n, l.np32);
    out[i] = uint4(x, 0);
}

kernel void copy(device const ulong4* src [[buffer(0)]], device ulong4* dst [[buffer(1)]], uint i [[thread_position_in_grid]]) {
    dst[i] = src[i];
}
"""

guard let device = MTLCreateSystemDefaultDevice(), let queue = device.makeCommandQueue() else {
    FileHandle.standardError.write("h2-gpu: no Metal device\n".data(using: .utf8)!)
    exit(1)
}
let library = try device.makeLibrary(source: source, options: nil)
func pipeline(_ name: String) -> MTLComputePipelineState {
    try! device.makeComputePipelineState(function: library.makeFunction(name: name)!)
}
let fillPSO = pipeline("fill"), applyPSO = pipeline("apply"), mulPSO = pipeline("mulmod"), copyPSO = pipeline("copy")

func load(_ name: String) -> Data {
    guard let d = FileManager.default.contents(atPath: dir + "/" + name) else {
        FileHandle.standardError.write("h2-gpu: \(dir)/\(name) missing (run h2-cpu --prepare)\n".data(using: .utf8)!)
        exit(1)
    }
    return d
}

func run(_ encode: (MTLComputeCommandEncoder) -> Void, blit: ((MTLBlitCommandEncoder) -> Void)? = nil) -> Double {
    let cb = queue.makeCommandBuffer()!
    if let blit = blit {
        let b = cb.makeBlitCommandEncoder()!
        blit(b)
        b.endEncoding()
    }
    let e = cb.makeComputeCommandEncoder()!
    encode(e)
    e.endEncoding()
    cb.commit()
    cb.waitUntilCompleted()
    if let err = cb.error {
        FileHandle.standardError.write("h2-gpu: \(err)\n".data(using: .utf8)!)
        exit(1)
    }
    return cb.gpuEndTime - cb.gpuStartTime
}

func median(_ v: [Double]) -> Double { v.sorted()[v.count / 2] }
func line(_ s: String) { print(s); fflush(stdout) }
print("# device: \(device.name)", to: &standardError)

// ---------------------------------------------------------------- sieve
let primes = load("primes.bin")
var items: [UInt32] = []
items.reserveCapacity(18_000_000)
var updates: UInt64 = 0
primes.withUnsafeBytes { (raw: UnsafeRawBufferPointer) in
    let w = raw.bindMemory(to: UInt32.self)
    let lineLen = UInt64(1) << UInt64(lineBits)
    for i in stride(from: 0, to: w.count, by: 2) {
        let p = UInt64(w[i]), r = UInt64(w[i + 1])
        updates += (lineLen - r + p - 1) / p
        var x = r
        while x < lineLen {
            items.append(UInt32(p))
            items.append(UInt32(x))
            x += itemUpdates * p
        }
    }
}
let nitems = UInt32(items.count / 2)
let itemBuf = device.makeBuffer(bytes: items, length: items.count * 4, options: .storageModeShared)!
let counts = device.makeBuffer(length: regions * 4, options: .storageModeShared)!
let upd = device.makeBuffer(length: regions * Int(cap) * 4, options: .storageModePrivate)!
let part = device.makeBuffer(length: regions * 4, options: .storageModeShared)!
var capV = cap, nitemsV = nitems
var sieveTimes: [Double] = []
for runIndex in 0...repeats {
    let fill = run({ e in
        e.setComputePipelineState(fillPSO)
        e.setBuffer(itemBuf, offset: 0, index: 0)
        e.setBuffer(counts, offset: 0, index: 1)
        e.setBuffer(upd, offset: 0, index: 2)
        e.setBytes(&capV, length: 4, index: 3)
        e.setBytes(&nitemsV, length: 4, index: 4)
        e.dispatchThreads(MTLSize(width: Int(nitems), height: 1, depth: 1),
                          threadsPerThreadgroup: MTLSize(width: min(256, fillPSO.maxTotalThreadsPerThreadgroup), height: 1, depth: 1))
    }, blit: { b in
        b.fill(buffer: counts, range: 0..<counts.length, value: 0)
        b.fill(buffer: part, range: 0..<part.length, value: 0)
    })
    let apply = run({ e in
        e.setComputePipelineState(applyPSO)
        e.setBuffer(upd, offset: 0, index: 0)
        e.setBuffer(counts, offset: 0, index: 1)
        e.setBuffer(part, offset: 0, index: 2)
        e.setBytes(&capV, length: 4, index: 3)
        e.setThreadgroupMemoryLength(1 << Int(regionBits + 1), index: 0)
        e.dispatchThreadgroups(MTLSize(width: regions, height: 1, depth: 1),
                               threadsPerThreadgroup: MTLSize(width: min(1024, applyPSO.maxTotalThreadsPerThreadgroup), height: 1, depth: 1))
    })
    let c = counts.contents().bindMemory(to: UInt32.self, capacity: regions)
    let s = part.contents().bindMemory(to: UInt32.self, capacity: regions)
    var total: UInt64 = 0, maxc: UInt32 = 0, sum: UInt64 = 0
    for i in 0..<regions {
        total += UInt64(c[i])
        maxc = max(maxc, c[i])
        sum += UInt64(s[i])
    }
    if maxc > cap {
        FileHandle.standardError.write("h2-gpu: a bucket overflowed (\(maxc) > \(cap))\n".data(using: .utf8)!)
        exit(1)
    }
    if total != updates {
        FileHandle.standardError.write("h2-gpu: \(total) updates counted, \(updates) expected\n".data(using: .utf8)!)
        exit(1)
    }
    let secs = fill + apply
    if runIndex > 0 { sieveTimes.append(secs) }
    line("sieve,gpu,\(runIndex),\(String(format: "%.6f", secs)),\(updates),\(String(format: "%.6e", Double(updates) / secs)),\(sum)")
    line("sieve-fill,gpu,\(runIndex),\(String(format: "%.6f", fill)),\(updates),\(String(format: "%.6e", Double(updates) / fill)),\(sum)")
    line("sieve-apply,gpu,\(runIndex),\(String(format: "%.6f", apply)),\(updates),\(String(format: "%.6e", Double(updates) / apply)),\(sum)")
}
print("# sieve: median \(String(format: "%.4f", median(sieveTimes))) s, \(nitems) work items, \(updates) updates", to: &standardError)

// ---------------------------------------------------------------- mulmod
let lanesData = load("lanes.bin")
let nlanes = lanesData.count / 112
let laneBuf = lanesData.withUnsafeBytes { device.makeBuffer(bytes: $0.baseAddress!, length: lanesData.count, options: .storageModeShared)! }
let outBuf = device.makeBuffer(length: nlanes * 16, options: .storageModeShared)!
var kV = K
var mulTimes: [Double] = []
for runIndex in 0...repeats {
    let secs = run { e in
        e.setComputePipelineState(mulPSO)
        e.setBuffer(laneBuf, offset: 0, index: 0)
        e.setBuffer(outBuf, offset: 0, index: 1)
        e.setBytes(&kV, length: 4, index: 2)
        e.dispatchThreads(MTLSize(width: nlanes, height: 1, depth: 1),
                          threadsPerThreadgroup: MTLSize(width: min(256, mulPSO.maxTotalThreadsPerThreadgroup), height: 1, depth: 1))
    }
    let ops = UInt64(nlanes) * UInt64(K)
    let o = outBuf.contents().bindMemory(to: UInt32.self, capacity: nlanes * 4)
    var check: UInt64 = 0
    for i in 0..<(nlanes * 4) { check = check &* 1_000_003 &+ UInt64(o[i]) }
    if runIndex > 0 { mulTimes.append(secs) }
    line("mulmod,gpu,\(runIndex),\(String(format: "%.6f", secs)),\(ops),\(String(format: "%.6e", Double(ops) / secs)),\(check)")
}
print("# mulmod: median \(String(format: "%.4f", median(mulTimes))) s over \(nlanes) lanes", to: &standardError)
let first = Data(bytes: outBuf.contents(), count: min(cpuLanes, nlanes) * 16)
try first.write(to: URL(fileURLWithPath: dir + "/gpu_lanes.bin"))

// ---------------------------------------------------------------- copy
let words = 1 << 27
let src = device.makeBuffer(length: words * 8, options: .storageModePrivate)!
let dst = device.makeBuffer(length: words * 8, options: .storageModePrivate)!
for runIndex in 0...repeats {
    let secs = run { e in
        e.setComputePipelineState(copyPSO)
        e.setBuffer(src, offset: 0, index: 0)
        e.setBuffer(dst, offset: 0, index: 1)
        e.dispatchThreads(MTLSize(width: words / 4, height: 1, depth: 1),
                          threadsPerThreadgroup: MTLSize(width: min(256, copyPSO.maxTotalThreadsPerThreadgroup), height: 1, depth: 1))
    }
    let bytes = UInt64(2 * words * 8)
    line("copy,gpu,\(runIndex),\(String(format: "%.6f", secs)),\(bytes),\(String(format: "%.6e", Double(bytes) / secs)),0")
}

