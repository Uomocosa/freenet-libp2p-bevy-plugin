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

One room channel: gossipsub, one topic per room. Joining a room (through
`discovery`) auto-subscribes the plugin to that room's topic.

- `p2p::Command::PublishInJoinedRoom { payload }` publishes to the room you have joined.
- `p2p::Command::PublishToSpecificRoom { room, payload }` publishes to a named room.
- Inbound payloads arrive as `p2p::Event::RoomMessage { room, from, payload }`.
- A payload larger than `p2p::GossipsubConfig::max_transmit_size` (64 KiB)
  is rejected with `p2p::Event::Error`, not silently dropped.
- `room_sync::SyncPlugin<Replica>` replicates a `RoomState` per room. Its
  `anti_entropy: Option<room_sync::AntiEntropyConfig>` (default on: 2 s interval, at
  most 8 slice fetches per peer; `with_anti_entropy(None)` turns it off) repairs ops
  lost on gossip: peers announce one content hash per replica slice and fetch the
  slices whose hash differs.

Defaults are tuned for small rooms (≤8 peers): 100 ms heartbeat, `mesh_n_low`
2, `flood_publish` enabled.

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
just contract   # build the lobby contract wasm
just freenet-harness   # freenet contract harness (CRDT laws) against the lobby contract
just setup      # one-time per clone: install the prek git hooks
just check      # run the full gate (same as CI)
just ci
```
