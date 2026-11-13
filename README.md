# freenet-libp2p-bevy-plugin

A generic Bevy P2P plugin: Freenet-backed room discovery plus libp2p Kademlia
peer history, with an optional built-in lobby UI.

## Using the plugin

Add it to your game's `Cargo.toml`. The plugin itself needs **no system
packages** — it pulls only the Bevy subcrates it uses:

```toml
[dependencies]
freenet-libp2p-bevy-plugin = { git = "https://github.com/Uomocosa/freenet-libp2p-bevy-plugin" }
```

- **Linux**: nothing for a networking-only build. If your game enables Bevy
  windowing/audio, install Bevy's standard deps
  (`libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev`).
- **macOS**: nothing beyond rustup (Metal/Cocoa/CoreAudio are native).
- **Windows**: nothing beyond rustup (DX12/Vulkan/WASAPI are native).
- GPU: uses the OS driver; headless falls back to WARP (Windows) / Metal
  (macOS) / lavapipe (Linux).

### Rooms

One room channel: gossipsub, one topic per room. Joining a room (through the
lobby-room `discovery`) auto-subscribes the plugin to that room's topic.

- `p2p::Command::PublishInJoinedRoom { payload }` publishes to the room you have joined.
- `p2p::Command::PublishToSpecificRoom { room, payload }` publishes to a named room.
- Inbound payloads arrive as `p2p::Event::RoomMessage { room, from, payload }`.
- A payload larger than `p2p::GossipsubConfig::max_transmit_size` (64 KiB)
  is rejected with `p2p::Event::Error`, not silently dropped.
- `room_sync::SyncPlugin<Replica>` replicates a `ReplicatedState` per room. Its
  `anti_entropy: Option<room_sync::AntiEntropyConfig>` (default on: 2 s interval, at
  most 8 slice fetches per peer; `with_anti_entropy_config(None)` turns it off) repairs ops
  lost on gossip: peers announce one content hash per replica slice and fetch the
  slices whose hash differs. Optional Kademlia room history is configured via
  `with_history_config(Some(room_sync::HistoryConfig::default()))`.

Defaults are tuned for small rooms (≤8 peers): 100 ms heartbeat, `mesh_n_low`
2, `flood_publish` enabled.

## Examples

- `examples/lobby_room/` — the smallest discovery + room-messaging app.
- `examples/starboard/` — the reference game: a shared 1024×1024 canvas over the
  room API, with a daily shape and colour per peer, presence cursors and
  anti-entropy repair. Run it with `just run-starboard --username <name>`; see
  [`examples/starboard/README.md`](examples/starboard/README.md).

## Workspace layout

- `crates/common_types/` — bincode wire types shared by the contract and the plugin.
- `crates/lobby_contract/` — the Freenet lobby contract (wasm via `just contract`).
- `crates/freenet-libp2p-bevy-core/` — p2p, room sync, testing harness (no discovery).
- `crates/freenet-libp2p-bevy-lobby-room/` — the lobby-room kit: Freenet lobby discovery, the Freenet client, and the default lobby UI (`LobbyRoomPlugins`, `ui::LobbyRoomUiPlugin`, `discovery::...`). Feature `testing`.
- `crates/freenet-libp2p-bevy-plugin/` — thin facade (default feature `lobby-room`, re-exported as `lobby_room`) over core and the lobby-room kit.
- `examples/*/` — runnable apps (`lobby_room`, `starboard`, `shared`, `e2e_check`, `tools`).
- `tests/e2e/` — the e2e suite (incl. `tests/e2e/tests/contract_wire.rs`); `tests/e2e_steps/` step definitions; `tests/e2e_macros/` test macros.

Depend on just `common_types` for the wire types, or on the whole `freenet-libp2p-bevy-plugin` facade for the plugin.

## Dependencies and releases

This crate depends on three sibling repositories at release tags
(`atomic-delegate-macros` from `lele-rust-linter`; `telegram-bot-helper` and
`lele-bevy-preview` from `telegram-bot-helper`; `freenet-contract-harness`):

    lele-bevy-preview = { git = "https://github.com/Uomocosa/telegram-bot-helper", tag = "v0.0.8" }

A git content filter (`lele-pins`) rewrites those pins to sibling `path =`
dependencies in your working tree, so an edit in a dependency is picked up by the
next `cargo build` here with no release and no command. Install it once per
machine:

    cd ../lele-release && just setup

Never hand-edit a pin. From the fleet root, `just release <patch|minor|major>`
bumps every changed repo, re-pins its dependents, tags `vX.Y.Z` and pushes; see
`lele-release/README.md` for `status`, `--only`/`--exclude` and `--dry-run`. A
plain clone without the filter resolves the committed tags, which is exactly what
CI sees.

## Developing this crate (examples, previews, e2e)

```sh
rustup toolchain install nightly        # rust-toolchain.toml pins channel + wasm target
cargo install cargo-nextest just
cargo install --git https://github.com/Uomocosa/lele-rust-linter lele-lint lele-bevy-lint
cargo binstall prek                     # hook runner (`prek install` via `just setup`)
```

System libraries for Bevy + headless previews + MP4 recording:

| OS | Install |
|---|---|
| Linux | `sudo apt install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev mesa-vulkan-drivers ffmpeg` |
| macOS | `brew install ffmpeg` |
| Windows | `winget install Gyan.FFmpeg` |

`target-dir = "/tmp/frt-build"` is set in `.cargo/config.toml` (the
space-in-path `tikv-jemalloc-sys` workaround), with `jobs = 6`.

## Tasks

```sh
just build | just clippy | just fmt | just test | just lint | just bevy-lint
just run          # lobby_room, detached; args pass through (--ui, --script, …)
just demo         # lobby_room with the default UI
just run-starboard --username <name>   # starboard, detached
just e2e-discovery | just e2e-join | just e2e-room | just e2e-starboard | just e2e-history
just bench-starboard   # rewrite examples/starboard/BENCH.md
just bench-history     # rewrite examples/starboard/HISTORY.md
just contract   # build the lobby contract wasm
just freenet-harness   # freenet contract harness (CRDT laws) against the lobby contract
just setup      # one-time per clone: install the prek git hooks
just check      # run the full gate (same as CI)
just ci
```
