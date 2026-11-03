# Primary Hardware Envelope

**Machine**: Mac M5 Max (Apple Silicon)

- Unified memory architecture (typical configurations 36–128 GB; exact installed capacity to be confirmed on the device).
- High core-count CPU and integrated GPU.
- No discrete NVIDIA GPU; all acceleration uses the Apple GPU / Metal or CPU.

## Usage rules for Primness
- All independent D1 measurements (polynomial evaluation, smoothness counting, Dickman comparisons) must complete on this machine within the < 1 hour wall-clock success criterion stated in the roadmap for the target size, or be explicitly marked as scaled-down.
- External tools (CADO-NFS, msieve, YAFU) are invoked only as instruments; their resource use is observed, not inherited as a limit.
- Docker containers may be used for reproducible builds or isolation of external instruments.
- Dashboards may be added to surface measurement results and CI status without introducing brittle gates.

## Relation to Obstacle D
Hardware capacity is part of the concrete resource limits against which any claimed smoothness excess must be measured. If a direction requires resources beyond this Mac, it is stated as a pivot toward Obstacle H rather than assumed available.
