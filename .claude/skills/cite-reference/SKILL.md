---
name: cite-reference
description: Record where a ported rule comes from — file:line and byte offset in the reference — in MECHANICS.md, using scripts/cite.py. Use after porting anything, and whenever a value needs proof.
---

# Cite the reference

1. Pick the exact Java text that holds the rule (a constant, a formula, an assignment).
2. Run `python scripts/cite.py <reference>/src/main/java <File.java> "<exact text>"`.
   It prints `jagex/File.java:LINE @ 0xOFFSET | line`. No match = don't cite it; recheck.
3. In `MECHANICS.md`:
   - describe the rule in the section of its feature (what it does, reference function, our module);
   - for key values add a row to the offset table:
     `| Rule in plain words (our name) | \`jagex/File.java\` @ 0xOFFSET | how it was found |`.
4. Make sure the build really uses that value (grep our code); if not, fix the code, not the doc.
