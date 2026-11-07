# Draft plan — leftovers from the E2E layers work

Draft only. Do not start this until the user reviews it, and not before `tests/DIRECT-DIAL-PLAN.md` is finished.

- Port the `lobby_room` tests to `tests/e2e/`, with `tests/lobby_room_steps/`.
- Add `e2e-local-gateways`: a private Freenet network on localhost.
- Port S2-S7 to cross-OS, using the checkpoint board from the direct dial plan.
- Remove the old `tests/*.rs` scenario targets and `tests/common/scenario.rs` once every scenario is ported.
- Decide what to do about the S6 false "member left" if it repeats.
