# Ponytail review lenses (vendored, MIT)

Source: <https://github.com/DietrichGebert/ponytail> (MIT; see `PONYTAIL-LICENSE`).

Vendored on purpose:

- `.opencode/skills/ponytail-{review,audit,debt,gain,help}/SKILL.md`
- `.opencode/command/ponytail-{review,audit,debt,gain,help}.md`

**Not vendored:** the always-on `ponytail` persona skill and the OpenCode plugin. This
project uses Ponytail as a one-shot review/audit lens, not an authoring rule set.

## Precedence

`lele-lint` > `lele-syntax-rs` (`RATIONALE.md` + `SMELLS.md`) > domain skill > Ponytail.

The global `~/.config/opencode/AGENTS.md` carries the binding precedence block. The
`lele-ponytail-bridge-rs` skill carries the Rust-specific translation.

In particular:

- Do not delete `#[atomic_delegates]` shells, `__basic__` types, single-field newtypes,
  `test_usage`, `thiserror` enums, or boundary structure because Ponytail calls them
  "lean" findings. RATIONALE §13 *Limits* and SMELLS S5 exempt them.
- Do not add `shortcut:` comments in `src/` or `__methods__/` (LELE031). Keep the ledger
  outside `src/`.
- Do not use Ponytail to clear lint errors; use `lele-lint-fix-rs`.
