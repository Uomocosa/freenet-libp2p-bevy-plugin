# Draft plan — leftovers from the E2E layers work

Draft only. Do not start this until the user reviews it, and not before `tests/DIRECT-DIAL-PLAN.md` is finished.

- Port the `lobby_room` tests to `tests/e2e/tests/`, with `tests/e2e_steps/` additions.
- Add `e2e-local-gateways`: a private Freenet network on localhost.
- Port S2-S7 to cross-OS, using the checkpoint board from the direct dial plan.
- Remove the old `tests/e2e/tests/*.rs` scenario targets and `tests/e2e/tests/common/scenario.rs` once every scenario is ported.
- Decide what to do about the S6 false "member left" if it repeats.

- `Orchestrator` always passes `--identity-file`: the `lobby_room` port must accept that flag too. Test identity files also pile up in the system temp dir; delete them when a run ends.
