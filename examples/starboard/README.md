# Starboard

A shared 1024×1024 drawing board built on this plugin: peers meet through the
Freenet lobby, join a room, and paint one canvas together. The board is a tiled
LWW CRDT; gossip is lossy and anti-entropy repairs what it loses.

See the root [`README.md`](../../README.md) for the plugin API and its defaults.

## Run

```sh
just run-starboard --username alice
```

The recipe builds the `starboard` example and starts it detached; its log is
`.local-run/starboard.log`. Direct equivalent:

```sh
cargo run --features e2e --example starboard -- --username alice
```

The app starts an embedded Freenet node and opens the plugin's default lobby UI:

1. Type a room name and press **Create New Room**, or click a room in the list.
2. A black **Joining room …** screen shows until the room is ready.
3. The board replaces the lobby. Paint.

To leave, click the door at the bottom-right or press **Esc**, then confirm.

| Argument | Meaning |
|---|---|
| `--username <name>` | required; your name in the member panel |
| `--token <game>` | lobby and topic namespace; defaults to `game_id!()` |
| `--transport tcp\|quic\|both` | libp2p transports (default `both`) |
| `--action create:<room>\|join:<room>` | drive the lobby with scripted input |
| `--script <step>,<step>,…` | drive the whole session with scripted steps |
| `--join-timeout-secs <n>` | how long to wait for the room to show up (default 180) |

`--action` and `--script` exist for the e2e harness; a normal run needs neither.
They are **exclusive**: passing both is a CLI error.

The `--script` steps are `create:<room>`, `join:<room>`, `leave:<room>`,
`wait-for-n-members:<n>`, `wait-secs:<n>`, `wait-converged:<room>`,
`paint-from-to:<x.y>-<x.y>`, `erase-from-to:<x.y>-<x.y>` and `exit`. A step that does
not make sense is rejected on the command line with the reason:

```
$ starboard --username p1 --script "paint-from-to:10.20-9999.5"
error: invalid value 'paint-from-to:10.20-9999.5' for '--script [<...>...]':
       cell '9999.5' is outside the canvas (0..1024)
```

The Kademlia history is on: every peer stores its room's board in the local
Kademlia store, so a peer that joins a room nobody is in still gets the board back
(`examples/starboard/HISTORY.md` has the measured sizes and latencies).

## Controls

| Input | Action |
|---|---|
| Left click / hold | stamp, then stroke, your daily shape |
| Right click / hold | erase with the same footprint |
| Scroll up / down | grow / shrink the brush (9 notches, 10 %–200 %) |

The brush is 16 cells at 100 %; it is local and per session. Your shape and colour
are assigned per day: `blake3(peer_id ‖ created_at ‖ day)` picks one of 6 shapes ×
12 colours, and colliding peers step to the next free combination in peer-id order.
Existing pixels keep their colour when the day rolls over.

The top-right panel lists the room's members with their icon and name. Every peer's
cursor is drawn in their own shape and colour at their reported brush size. Presence
is lossy and latest-wins: a peer that stops moving keeps its last cursor until it
leaves.

## Architecture

```text
Freenet lobby contract (contract/lobby, src/discovery)
        │  room name, members, created_at
        ▼
discovery::Status ──joined──► p2p gossipsub room topic (src/p2p)
        │                         │ CursorUpdate, ~30 Hz, lossy
        │                         ▼
        └──────────────► room_sync::SyncPlugin<Canvas> (src/room_sync)
                                  │ ops + Lamport clock
                                  │ join snapshot
                                  │ anti-entropy per replica slice
                                  ▼
                          starboard::canvas::Canvas
                                  │ dirty tiles
                                  ▼
                 starboard::render (one 1024×1024 Image) + starboard::ui
```

The plugin owns discovery, transport and sync; the example owns the game:

| Module | Responsibility |
|---|---|
| `canvas/` | dense 1024×1024 grid, 32×32 cells per tile; `RoomState` (`apply`, `merge`, slice hashes) |
| `identity/` | daily shape/colour from the lobby's `created_at`, 72 combinations |
| `input/` | pointer → `DrawOp`, brush-size notches, `PendingLocalDraw` |
| `render/` | letterboxed board texture, dirty-tile repaint, brush outline, remote cursors |
| `room/` | room brushes, presence publishing, cursor collection, canvas reset on join |
| `ui/` | member panel |
| `app/` | `StarboardPlugin`, the ordered system chain, the test harness |
| `bench/` | `#[cfg(test)]` benchmark behind `just bench-starboard` |

## Tests

```sh
just test              # unit tests and UI previews
just e2e-starboard     # 3 real windows + a late joiner over Freenet (X + network)
just e2e-history       # the Kademlia history scenarios S4a, S4b, S4c (X + network)
just bench-starboard   # 2 and 8 in-process peers; rewrites BENCH.md
just bench-history     # one room's history size and put/get latency; rewrites HISTORY.md
```

[`BENCH.md`](BENCH.md) holds the last measured numbers of the room sync and the limits
of the measurement; [`HISTORY.md`](HISTORY.md) holds the measured Kademlia history
sizes and latencies. [`PLAN.md`](PLAN.md) and [`PLAN_HISTORY.md`](PLAN_HISTORY.md) hold
the step history and the Phase 2 design.

## Wire format

- A cell is `{ lamport: u32, author: u16, color: Option<u8> }`; merge keeps the
  greatest `(lamport, author, color)`, with `None` (erased) below `Some(_)`.
- Ops travel on the room topic; join snapshots and slice repairs travel on the
  plugin's room-sync request/response protocol.
- A replica slice is one 32×32 tile; its content hash is 8 bytes of blake3.
- The 1024×1024 logical canvas is letterboxed into the window.
