#!/usr/bin/env python3
"""Apply the current-state contract named by the head commit.

A contract is a JSON Schema under schemas/ whose properties fix every
requirement with const. Enums are the closed options. This program
materializes those consts and interprets them. The field names below are
the interpreter vocabulary. The values are not.
"""

import copy
import json
import subprocess
import sys
from pathlib import Path

# Present together, these consts mark a schema as a current-state contract.
# schema_version alone does not: measurement-event schemas use that too.
CONTRACT_MARKERS = (
    "role",
    "version_metadata_label",
    "roadmap_path",
)
SELECTOR_FIELDS = ("schema_version",) + CONTRACT_MARKERS

# Read from every coexisting contract before the head document can name one.
POINTER_FIELDS = (
    "version_metadata_label",
    "roadmap_path",
    "metadata_line_prefix",
    "metadata_label_suffix",
)

# Fields this interpreter knows how to enforce. A contract that sets any
# other field, or omits one of these, is rejected.
IMPLEMENTED = {
    "schema_version",
    "role",
    "version_metadata_label",
    "roadmap_path",
    "history_path",
    "companion_paths",
    "section_heading",
    "section_ends",
    "stamp_prefix",
    "stamp_suffix",
    "history_heading_prefix",
    "keep_previous_history_heading",
    "metadata_line_prefix",
    "metadata_label_suffix",
    "roadmap_labels_must_change",
    "roadmap_labels_required_on_head",
    "history_labels_must_change",
    "history_labels_required_on_head",
    "require_section_changed",
    "require_stamp_changed",
}

# section_ends[].match values this interpreter implements.
# A contract enum must be a subset of this set.
MATCH_KINDS = {"prefix", "exact"}

ANNOTATIONS = {
    "description",
    "title",
    "$schema",
    "$id",
    "$comment",
    "default",
    "examples",
}
SCHEMA_KEYWORDS = ANNOTATIONS | {
    "type",
    "const",
    "enum",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "minItems",
    "maxItems",
    "uniqueItems",
    "minLength",
    "maxLength",
    "minimum",
    "maximum",
}

QUIET = False


def fail(message):
    if not QUIET:
        print(f"current-state check: {message}", file=sys.stderr)
    raise SystemExit(1)


def git_bytes(args):
    return subprocess.run(args, capture_output=True)


def git_show(ref, path):
    result = git_bytes(["git", "show", f"{ref}:{path}"])
    if result.returncode != 0:
        fail(f"{ref} is missing {path}")
    return result.stdout.decode("utf-8")


def list_schema_paths(ref):
    result = git_bytes(["git", "ls-tree", "-r", "--name-only", ref, "schemas"])
    if result.returncode != 0:
        fail(f"{ref} has no schemas tree")
    text = result.stdout.decode("utf-8")
    return [line for line in text.splitlines() if line.endswith(".json")]


def meta_value(text, label, prefix, suffix):
    needle = f"{prefix}{label}{suffix}"
    for line in text.splitlines():
        if line.startswith(needle):
            return line[len(needle) :]
    return None


def line_ends(line, ends):
    for end in ends:
        kind = end["match"]
        text = end["text"]
        if kind == "prefix":
            if line.startswith(text):
                return True
        elif kind == "exact":
            if line == text:
                return True
        else:
            fail(f"unsupported section end match {kind!r}")
    return False


def section_lines(text, heading, ends):
    lines = text.splitlines()
    try:
        start = lines.index(heading)
    except ValueError:
        return None
    body = []
    for line in lines[start + 1 :]:
        if line_ends(line, ends):
            break
        body.append(line)
    return body


def section_text(text, heading, ends):
    body = section_lines(text, heading, ends)
    if body is None:
        return None
    if not body:
        return ""
    return "\n".join(body) + "\n"


def stamp_of(body, prefix, suffix):
    if body is None:
        return None
    for line in body.splitlines():
        if not line.startswith(prefix) or not line.endswith(suffix):
            continue
        if len(line) <= len(prefix) + len(suffix):
            continue
        return line[len(prefix) : len(line) - len(suffix)]
    return None


def check_type(expected, instance, path):
    if not isinstance(expected, str):
        fail(f"{path} schema type must be a single string")
    checks = {
        "object": lambda value: isinstance(value, dict),
        "array": lambda value: isinstance(value, list),
        "string": lambda value: isinstance(value, str),
        "boolean": lambda value: isinstance(value, bool),
        "integer": lambda value: isinstance(value, int) and not isinstance(value, bool),
        "number": lambda value: isinstance(value, (int, float)) and not isinstance(value, bool),
    }
    if expected not in checks:
        fail(f"{path} uses unsupported type {expected}")
    if not checks[expected](instance):
        fail(f"{path} expected type {expected}")


def validate_node(schema, instance, path):
    if not isinstance(schema, dict):
        fail(f"{path} schema node is not an object")
    unknown = sorted(set(schema) - SCHEMA_KEYWORDS)
    if unknown:
        fail(f"{path} uses unsupported schema keywords: {', '.join(unknown)}")
    if "type" in schema:
        check_type(schema["type"], instance, path)
    if "enum" in schema:
        if not isinstance(schema["enum"], list) or instance not in schema["enum"]:
            fail(f"{path} value {instance!r} is outside the enum")
    if "const" in schema and instance != schema["const"]:
        fail(f"{path} value does not equal const")
    if "minLength" in schema or "maxLength" in schema:
        if not isinstance(instance, str):
            fail(f"{path} length constraint on a non-string")
        if "minLength" in schema and len(instance) < schema["minLength"]:
            fail(f"{path} is shorter than minLength")
        if "maxLength" in schema and len(instance) > schema["maxLength"]:
            fail(f"{path} is longer than maxLength")
    if "minimum" in schema or "maximum" in schema:
        if isinstance(instance, bool) or not isinstance(instance, (int, float)):
            fail(f"{path} numeric bound on a non-number")
        if "minimum" in schema and instance < schema["minimum"]:
            fail(f"{path} is below minimum")
        if "maximum" in schema and instance > schema["maximum"]:
            fail(f"{path} is above maximum")
    if "items" in schema or "minItems" in schema or "maxItems" in schema or "uniqueItems" in schema:
        if not isinstance(instance, list):
            fail(f"{path} expected an array")
        if "minItems" in schema and len(instance) < schema["minItems"]:
            fail(f"{path} has fewer than minItems")
        if "maxItems" in schema and len(instance) > schema["maxItems"]:
            fail(f"{path} has more than maxItems")
        if schema.get("uniqueItems"):
            canned = [json.dumps(item, sort_keys=True, separators=(",", ":")) for item in instance]
            if len(set(canned)) != len(canned):
                fail(f"{path} has duplicate items")
        if "items" in schema:
            for index, item in enumerate(instance):
                validate_node(schema["items"], item, f"{path}[{index}]")
    if "properties" in schema or "required" in schema or schema.get("additionalProperties") is False:
        if not isinstance(instance, dict):
            fail(f"{path} expected an object")
        properties = schema.get("properties", {})
        if not isinstance(properties, dict):
            fail(f"{path} properties is not an object")
        additional = schema.get("additionalProperties", True)
        if additional is False:
            extra = sorted(set(instance) - set(properties))
            if extra:
                fail(f"{path} has properties outside the schema: {', '.join(extra)}")
        elif additional is not True:
            fail(f"{path} additionalProperties must be true or false")
        for key in schema.get("required", []):
            if key not in instance:
                fail(f"{path} is missing required {key}")
        for key, value in instance.items():
            if key in properties:
                validate_node(properties[key], value, f"{path}.{key}")


def classify(schema, where):
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        return False
    markers = [
        key
        for key in CONTRACT_MARKERS
        if isinstance(properties.get(key), dict) and "const" in properties[key]
    ]
    if not markers:
        return False
    version = properties.get("schema_version")
    if len(markers) != len(CONTRACT_MARKERS) or not (
        isinstance(version, dict) and "const" in version
    ):
        fail(f"{where} has some contract selector fields but not all of {', '.join(SELECTOR_FIELDS)}")
    if schema.get("$id") != version["const"]:
        fail(f"{where} $id does not equal schema_version const")
    return True


def materialize(schema, where):
    properties = schema.get("properties")
    required = schema.get("required")
    if not isinstance(properties, dict) or not isinstance(required, list):
        fail(f"{where} contract schema needs properties and required")
    if set(properties) != set(required):
        fail(f"{where} required list and properties must name the same fields")
    instance = {}
    for key in required:
        prop = properties[key]
        if not isinstance(prop, dict) or "const" not in prop:
            fail(f"{where} does not fix {key} with const")
        instance[key] = copy.deepcopy(prop["const"])
    return instance


def check_capabilities(schema, instance):
    schema_id = instance["schema_version"]
    extra = sorted(set(instance) - IMPLEMENTED)
    missing = sorted(IMPLEMENTED - set(instance))
    if extra:
        fail(f"{schema_id} sets {', '.join(extra)}, which this checker does not implement")
    if missing:
        fail(f"{schema_id} does not set {', '.join(missing)}")
    match_enum = (
        schema.get("properties", {})
        .get("section_ends", {})
        .get("items", {})
        .get("properties", {})
        .get("match", {})
        .get("enum")
    )
    if not isinstance(match_enum, list) or not match_enum:
        fail(f"{schema_id} section_ends match enum is missing")
    unknown = [value for value in match_enum if value not in MATCH_KINDS]
    if unknown:
        fail(
            f"{schema_id} match enum includes {unknown}, which this checker does not implement"
        )
    label = instance["version_metadata_label"]
    if label not in instance["roadmap_labels_required_on_head"]:
        fail(f"{schema_id} must require its version metadata label {label!r} on the head document")


def load_contracts(ref):
    contracts = []
    for path in list_schema_paths(ref):
        where = f"{ref}:{path}"
        try:
            schema = json.loads(git_show(ref, path))
        except json.JSONDecodeError as err:
            fail(f"{where} is not JSON: {err}")
        if not isinstance(schema, dict):
            fail(f"{where} is not a JSON object")
        if not classify(schema, where):
            continue
        instance = materialize(schema, where)
        validate_node(schema, instance, where)
        check_capabilities(schema, instance)
        contracts.append(instance)
    if not contracts:
        fail(f"{ref} has no current-state contract schema under schemas/")
    pointers = {tuple(instance[key] for key in POINTER_FIELDS) for instance in contracts}
    if len(pointers) != 1:
        names = ", ".join(instance["schema_version"] for instance in contracts)
        fail(
            "coexisting current-state contracts ("
            + names
            + ") disagree on the version pointer (label, roadmap path, or metadata line shape)"
        )
    by_id = {}
    for instance in contracts:
        schema_id = instance["schema_version"]
        if schema_id in by_id:
            fail(f"duplicate current-state schema id {schema_id}")
        by_id[schema_id] = instance
    return by_id


def select_contract(ref):
    by_id = load_contracts(ref)
    sample = next(iter(by_id.values()))
    roadmap = git_show(ref, sample["roadmap_path"])
    chosen = meta_value(
        roadmap,
        sample["version_metadata_label"],
        sample["metadata_line_prefix"],
        sample["metadata_label_suffix"],
    )
    if chosen is None or chosen == "":
        fail(
            f"{ref} {sample['roadmap_path']} metadata has no {sample['version_metadata_label']} line"
        )
    if chosen not in by_id:
        known = ", ".join(sorted(by_id))
        fail(
            f"{ref} names schema {chosen}, which is not a current-state contract in that commit. Known: {known}"
        )
    return by_id[chosen]


def require_labels(text, labels, prefix, suffix, ref, what):
    for label in labels:
        if meta_value(text, label, prefix, suffix) is None:
            fail(f"{ref} {what} metadata has no {label} line")


def require_changed(base_text, head_text, labels, prefix, suffix, base, what):
    for label in labels:
        base_value = meta_value(base_text, label, prefix, suffix)
        head_value = meta_value(head_text, label, prefix, suffix)
        if base_value is None:
            fail(f"{base} {what} metadata has no {label} line")
        if head_value is None:
            fail(f"head {what} metadata has no {label} line")
        if base_value == head_value:
            fail(f"{what} metadata {label} is unchanged from {base}")


def exact_line(text, line):
    return line in text.splitlines()


def check(base, head):
    contract = select_contract(head)
    heading = contract["section_heading"]
    ends = contract["section_ends"]
    roadmap_path = contract["roadmap_path"]
    history_path = contract["history_path"]
    prefix = contract["metadata_line_prefix"]
    suffix = contract["metadata_label_suffix"]

    base_road = git_show(base, roadmap_path)
    head_road = git_show(head, roadmap_path)
    base_state = section_text(base_road, heading, ends)
    head_state = section_text(head_road, heading, ends)
    if base_state is None:
        fail(f"{base} has no {heading} section in {roadmap_path}")
    if head_state is None:
        fail(f"{head} has no {heading} section in {roadmap_path}")
    if contract["require_section_changed"] and base_state == head_state:
        fail(
            f"{roadmap_path} section {heading} is unchanged from {base}. "
            f"Update that section and record the snapshot in {history_path} "
            "before Auto-PR will open or refresh the draft."
        )

    base_as = stamp_of(base_state, contract["stamp_prefix"], contract["stamp_suffix"])
    head_as = stamp_of(head_state, contract["stamp_prefix"], contract["stamp_suffix"])
    if base_as is None:
        fail(f"{base} section has no stamp line")
    if head_as is None:
        fail(f"{head} section has no stamp line")
    if contract["require_stamp_changed"] and base_as == head_as:
        fail(f"the section stamp is unchanged from {base}")

    require_changed(
        base_road,
        head_road,
        contract["roadmap_labels_must_change"],
        prefix,
        suffix,
        base,
        "roadmap",
    )
    require_labels(
        head_road,
        contract["roadmap_labels_required_on_head"],
        prefix,
        suffix,
        head,
        "roadmap",
    )

    base_hist = git_show(base, history_path)
    head_hist = git_show(head, history_path)
    heading_prefix = contract["history_heading_prefix"]
    if not exact_line(head_hist, heading_prefix + head_as):
        fail(f"{history_path} is missing the new snapshot heading: {heading_prefix}{head_as}")
    if contract["keep_previous_history_heading"] and not exact_line(
        head_hist, heading_prefix + base_as
    ):
        fail(
            f"{history_path} no longer contains the previous snapshot heading: {heading_prefix}{base_as}"
        )
    require_changed(
        base_hist,
        head_hist,
        contract["history_labels_must_change"],
        prefix,
        suffix,
        base,
        "history",
    )
    require_labels(
        head_hist,
        contract["history_labels_required_on_head"],
        prefix,
        suffix,
        head,
        "history",
    )

    for path in contract["companion_paths"]:
        result = git_bytes(["git", "diff", "--quiet", base, head, "--", path])
        if result.returncode == 0:
            fail(
                f"{path} is unchanged from {base}. The current-state contract requires this file to change with the roadmap snapshot."
            )
        if result.returncode != 1:
            fail(f"git diff failed for {path}")

    print(f"current-state check: {base_as} -> {head_as}")


def print_section(ref):
    contract = select_contract(ref)
    text = git_show(ref, contract["roadmap_path"])
    lines = text.splitlines()
    try:
        start = lines.index(contract["section_heading"])
    except ValueError:
        return
    output = [contract["section_heading"]]
    for line in lines[start + 1 :]:
        if line_ends(line, contract["section_ends"]):
            break
        output.append(line)
    sys.stdout.write("\n".join(output) + "\n")


def main(argv):
    if argv and argv[0] == "--print-section":
        if len(argv) > 2:
            fail("usage: check_current_state.py --print-section [head-ref]")
        print_section(argv[1] if len(argv) == 2 else "HEAD")
        return
    if len(argv) > 2:
        fail("usage: check_current_state.py [base-ref] [head-ref]")
    base = argv[0] if len(argv) >= 1 else "origin/main"
    head = argv[1] if len(argv) == 2 else "HEAD"
    check(base, head)


REPO = Path(__file__).resolve().parents[1]


def self_test():
    global QUIET
    schema_path = REPO / "schemas" / "current-state" / "1.json"
    measurement_path = REPO / "schemas" / "measurement-event" / "1.json"
    schema = json.loads(schema_path.read_text())
    instance = materialize(schema, str(schema_path))
    validate_node(schema, instance, str(schema_path))
    check_capabilities(schema, instance)
    assert not classify(json.loads(measurement_path.read_text()), "measurement")

    broken = copy.deepcopy(schema)
    broken["properties"]["section_ends"]["items"]["properties"]["match"]["enum"] = [
        "prefix",
        "exact",
        "regex",
    ]
    QUIET = True
    try:
        check_capabilities(broken, instance)
    except SystemExit:
        pass
    else:
        raise SystemExit("regex match enum was accepted")
    extra = copy.deepcopy(schema)
    extra["properties"]["frob"] = {"const": 1}
    extra["required"] = list(extra["required"]) + ["frob"]
    try:
        check_capabilities(extra, materialize(extra, "frob"))
    except SystemExit:
        pass
    else:
        raise SystemExit("unknown contract field was accepted")
    QUIET = False

    import tempfile

    stamp_a = "2026-11-03 15:58 EST"
    stamp_b = "2026-11-03 16:17 EST"

    def meta(modified, schema_id=None):
        lines = [
            "# Road",
            "",
            "**Metadata**",
            f"- **Last Modified**: {modified}",
        ]
        if schema_id is not None:
            lines.append(f"- **Schema**: {schema_id}")
        lines.append("")
        return "\n".join(lines)

    def section(stamp, body):
        return (
            "## Current State\n\n"
            f"As of {stamp}:\n\n"
            f"{body}\n\n"
            "---\n\n"
            "SENTINEL\n"
        )

    def history(modified, stamps):
        headings = "\n".join(f"### {stamp}\n- note\n" for stamp in stamps)
        return (
            "# History\n\n"
            "**Metadata**\n"
            f"- **Last Modified**: {modified}\n\n"
            "## Current State Entries\n\n"
            f"{headings}"
        )

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)

        def write(rel, text):
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)

        def git(*args):
            result = subprocess.run(args, cwd=root, capture_output=True, text=True)
            if result.returncode != 0:
                raise SystemExit(result.stderr or result.stdout)
            return result.stdout.strip()

        def run(*args):
            return subprocess.run(
                [sys.executable, str(Path(__file__).resolve()), *args],
                cwd=root,
                capture_output=True,
                text=True,
            )

        git("git", "init", "-q", "-b", "main")
        git("git", "config", "user.email", "current-state-test@localhost")
        git("git", "config", "user.name", "current-state-test")
        write("schemas/current-state/1.json", schema_path.read_text())
        write("schemas/measurement-event/1.json", measurement_path.read_text())
        write("primness_roadmap.md", meta(stamp_a) + section(stamp_a, "base snapshot"))
        write("context_convo_notes/current_state_history.md", history(stamp_a, [stamp_a]))
        write("README.md", "base readme\n")
        git("git", "add", ".")
        git("git", "commit", "-q", "-m", "base")
        base = git("git", "rev-parse", "HEAD")

        write(
            "primness_roadmap.md",
            meta(stamp_b, "current-state/1") + section(stamp_b, "head snapshot"),
        )
        write(
            "context_convo_notes/current_state_history.md",
            history(stamp_b, [stamp_b, stamp_a]),
        )
        write("README.md", "head readme\n")
        git("git", "add", ".")
        git("git", "commit", "-q", "-m", "head")
        head = git("git", "rev-parse", "HEAD")

        ok = run(base, head)
        if ok.returncode != 0:
            raise SystemExit(ok.stderr)
        if f"{stamp_a} -> {stamp_b}" not in ok.stdout:
            raise SystemExit(ok.stdout)
        printed = run("--print-section", head)
        if printed.returncode != 0 or "SENTINEL" in printed.stdout:
            raise SystemExit(printed.stdout + printed.stderr)
        if not printed.stdout.startswith("## Current State\n"):
            raise SystemExit(printed.stdout)

        def expect_fail(mutator, needle):
            git("git", "reset", "--hard", "-q", head)
            mutator()
            git("git", "add", ".")
            git("git", "commit", "-q", "-m", "variant")
            result = run(base, "HEAD")
            git("git", "reset", "--hard", "-q", head)
            if result.returncode == 0 or needle not in result.stderr:
                raise SystemExit(
                    f"expected failure containing {needle!r}\n{result.stdout}\n{result.stderr}"
                )

        def drop_previous():
            write(
                "context_convo_notes/current_state_history.md",
                history(stamp_b, [stamp_b]),
            )

        def same_section():
            text = (root / "primness_roadmap.md").read_text().replace(
                "head snapshot", "base snapshot"
            ).replace(f"As of {stamp_b}:", f"As of {stamp_a}:")
            # Keep Last Modified changed so the section check is the one that fires.
            write("primness_roadmap.md", text)

        def same_readme():
            write("README.md", "base readme\n")

        def unknown_schema():
            text = (root / "primness_roadmap.md").read_text().replace(
                "current-state/1", "current-state/99"
            )
            write("primness_roadmap.md", text)

        def disagreeing_contract():
            other = json.loads(schema_path.read_text())
            other["$id"] = "current-state/2"
            other["properties"]["schema_version"]["const"] = "current-state/2"
            other["properties"]["metadata_line_prefix"]["const"] = "- "
            write("schemas/current-state/2.json", json.dumps(other))

        expect_fail(drop_previous, stamp_a)
        expect_fail(same_section, "unchanged")
        expect_fail(same_readme, "README.md")
        expect_fail(unknown_schema, "current-state/99")
        expect_fail(disagreeing_contract, "disagree")

    print("current-state self-test: ok")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        self_test()
    else:
        main(sys.argv[1:])
