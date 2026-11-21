#!/usr/bin/env python3
"""Writes THIRD_PARTY_NOTICES.md: the crates the engine is built from, and
their licences.

    cargo deny list --format json --layout crate > build/deny-list.json
    cargo metadata --format-version 1 --locked > build/metadata.json
    python3 tools/third-party-notices.py build/deny-list.json build/metadata.json > THIRD_PARTY_NOTICES.md

The crates are cargo-deny's (deny.toml's graph: every platform and feature,
no development-only crates); each one's licence expression and source are
Cargo's. CI runs this and fails when the file in the repository differs.
Standard library only, so it runs on any CI machine's Python 3.
"""

from __future__ import annotations

import json
import sys

HEAD = """\
# Third-party notices

The brush engine is built from the crates below, each under its own licence:
the licence expression its authors declare, as Cargo reads it. This
repository holds none of their code; Cargo fetches them, at the versions in
`Cargo.lock`, when it builds. A program built from it (`brush`,
`brush_host.dll`) contains some of them, and whoever passes such a program
on passes on their licence texts and notices with it.

Every licence here is one `deny.toml` allows (MIT, Apache-2.0, BSD-2-Clause,
ISC, Zlib, Unicode-3.0), and each may be combined with this repository's
Apache-2.0. The list covers every platform and feature, not only what one
build uses. Tools used only in development aren't in any program built from
here and aren't listed: cbindgen (MPL-2.0) writes `include/brush_host.h` in
a test.

Written by `tools/third-party-notices.py` from `cargo deny list` and
`cargo metadata`; CI checks it is current.

| Crate | Version | Licence | Source |
|---|---|---|---|
"""


def notices(deny_list: dict[str, object], metadata: dict[str, object]) -> str:
    """The file's text, from cargo-deny's crate list and cargo's metadata."""
    packages = {(p["name"], p["version"]): p for p in metadata["packages"] if p.get("source")}  # type: ignore[index,union-attr]
    rows = []
    for key in deny_list:
        name, version, source = key.split(" ", 2)
        if not source.startswith("registry+"):
            continue  # this workspace's own crates
        package = packages[(name, version)]
        licence = package.get("license") or f"see {package.get('license_file')}"
        where = package.get("repository") or f"https://crates.io/crates/{name}"
        rows.append((name, version, licence, where))
    rows.sort()
    return HEAD + "".join(f"| {n} | {v} | {lic} | <{w}> |\n" for n, v, lic, w in rows)


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    with open(argv[0], encoding="utf-8-sig") as f:
        deny_list = json.load(f)
    with open(argv[1], encoding="utf-8-sig") as f:
        metadata = json.load(f)
    sys.stdout.reconfigure(newline="\n")  # type: ignore[union-attr]
    sys.stdout.write(notices(deny_list, metadata))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
