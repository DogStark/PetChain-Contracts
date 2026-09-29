#!/usr/bin/env python3
"""Extracts the public contract ABI (exported fn signatures) from
stellar-contracts/src/lib.rs and prints a normalized, sorted snapshot.

Used by `generate_abi_snapshot.sh` and by CI (see
.github/workflows/stellar-contracts.yml, job `abi-snapshot`) to detect
unreviewed changes to the contract's public interface. See
stellar-contracts/docs/abi-migrations.md for the process to follow when a
change is intentional.
"""
import re
import sys

def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "src/lib.rs"
    with open(path) as f:
        text = f.read()

    marker = "#[contractimpl]"
    start = text.index(marker)
    # Find the `impl ... {` that immediately follows the attribute, then
    # walk forward balancing braces to find the end of that impl block.
    brace_start = text.index("{", start)
    depth = 0
    i = brace_start
    while True:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    body = text[brace_start + 1 : i]

    sigs = []
    for m in re.finditer(r"\bpub fn\s+(\w+)\s*\(", body):
        name = m.group(1)
        paren_start = m.end() - 1
        depth = 0
        j = paren_start
        while True:
            if body[j] == "(":
                depth += 1
            elif body[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        args = body[paren_start + 1 : j]
        rest = body[j + 1 : j + 300]
        ret_match = re.match(r"\s*->\s*([^\{]+)\{", rest)
        ret = ret_match.group(1).strip() if ret_match else ""

        def norm(s):
            return re.sub(r"\s+", " ", s).strip()

        args_norm = norm(args)
        sig = f"pub fn {name}({args_norm})"
        if ret:
            sig += f" -> {norm(ret)}"
        sigs.append(sig)

    for sig in sorted(sigs):
        print(sig)

if __name__ == "__main__":
    main()
