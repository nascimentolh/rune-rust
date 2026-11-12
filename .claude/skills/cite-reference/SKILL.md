---
name: cite-reference
description: Record where a ported rule comes from — file:line and byte offset in the reference — in MECHANICS.md, using scripts/cite.py, and confirm the build uses that exact value. Use after porting anything, and whenever a value needs proof.
---

# Cite the reference

1. Pick the exact Java text that holds the rule: the constant, the formula, the assignment.
   Copy it exactly, spaces included (the reference is auto-formatted, so it's stable).
2. Run:
   ```sh
   python scripts/cite.py <reference>/src/main/java Class338.java "static int[] anIntArray3973 = {256, 128, 86, 64};"
   # jagex/Class338.java:5 @ 0x31  |  static int[] anIntArray3973 = {256, 128, 86, 64};
   ```
   - No match: the text differs (spaces, `i_3`-style names) or the rule isn't there. Don't cite.
   - Several matches: cite the one inside the function you ported.
3. In `MECHANICS.md`:
   - in the feature's section, a table row or sentence: what the rule is, the reference function
     (`Class338.method6025`), our module and function;
   - for key values, a row in the offset table near the top:
     ``| Floor-1 ranges 256, 128, 86, 64 (`FLOOR_RANGES`) | `jagex/Class338.java` @ 0x31 | cite.py |``.
4. Confirm the build uses it: `grep -rn "256, 128, 86, 64" src/`. If the code differs from the
   reference, fix the code (and its test), never the citation.
5. Say plainly in the section, and in `ROADMAP.md`, what part of the feature isn't ported yet.
