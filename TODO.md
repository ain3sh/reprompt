- [x] Understand and reproduce the new prompt restoration failure case locally (note symptoms and trigger conditions).
- [x] Review AGENT_REMINDERS.md
- [x] Trace the restore logic end-to-end to locate encoding/algorithm bugs; jot hypotheses here.
- [x] Review AGENT_REMINDERS.md
- [x] Design and implement a robust prompt restoration strategy (avoid duct-tape fixes); document reasoning in code/tests.
- [x] Review AGENT_REMINDERS.md
- [x] Add/extend tests covering the failure case and edge scenarios; ensure they exercise real code paths.
- [x] Review AGENT_REMINDERS.md
- [x] Verify the full test suite passes and the failure is resolved.
- [x] Review AGENT_REMINDERS.md

Scratchpad notes:
- The failure reproduced as mojibake-heavy borders that the original regex ignored; top/bottom lines slipped through and ∩┐╜ tokens stayed in-line.
- Implemented cp1252 round-trip recovery and inline border scrubbing to rehabilitate corrupted glyphs before stripping wrappers.
