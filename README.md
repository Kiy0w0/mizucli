# 水 mizu

> A  TUI system monitor written in Rust

`mizu` (水, *water*) is a terminal system monitor that turns your machine's
CPU, RAM, disk, and network activity into a calm, flowing deep-sea panel.
The center "flow" panel plays **Bad Apple!!** as a live ASCII animation
while metrics stream around it. Built on
[ratatui](https://github.com/ratatui/ratatui) with an Elm-style
architecture — metrics collection, animation ticks, and rendering stay
cleanly separated.

```
 水 mizu  ~ fluid system monitor ~        CPU: 13th Gen Intel(R) Core(TM) i7   GPU: NVIDIA RTX 4060
 ┌─ CPU  13th Gen Intel… ────┐  ┌─ RAM ─────────────────────┐
 │ c00 ▓▓▓▓▓▓▓░░  42%        │  │ 3.6 GiB / 16.0 GiB (22%)  │
 │ c01 ▓▓▓░░░░░░  18%        │  └───────────────────────────┘
 │ ...                       │  ┌─ Network ─────────────────┐
 └───────────────────────────┘  │ ▼ rx     124.3 KB/s       │
 ┌─ ~ flow ~ ─────────────────────────────────────────────┐
 │ ############     ##############     ###########       │
 │ ###    ####     ####        ####    ####    ####      │
 │ ###    ####     ##############      ###########       │
 │ ############     ####                ####    ####     │
 └────────────────────────────────────────────────────────┘
 ┌─ Disk ─────────────────────────────────────────────────┐
 │ C:\  224.1 GiB / 953.8 GiB  24%                        │
 └────────────────────────────────────────────────────────┘
 [1] overview  [2] cpu  [3] net/disk  t:mizu  f:flow  q quit
```

## Features

- **Bad Apple flow panel** — 6572 RLE-encoded frames (80×45) decoded and
  stretched edge-to-edge as `#`/space ASCII art. No audio, just clean
  visuals. Toggle live with `f`.
- **Async metrics core** — a Tokio background task samples `sysinfo` once a
  second and ships snapshots through an `mpsc` channel; the UI loop never
  blocks on I/O.
- **Hardware identity** — CPU brand string (via `sysinfo`) and GPU name
  (via Windows PowerShell `Get-CimInstance Win32_VideoController`) shown in
  the header, Task-Manager style.
- **Per-core CPU grid** — auto-laid-out gauges, one per logical core,
  colour-graded teal → cyan → coral.
- **RAM, disk, network** — live RAM gauge, per-disk usage bars with disk
  I/O sparklines (aggregated per-process read/write via sysinfo), and
  `rx`/`tx` network sparklines with a rolling 60-sample history.
- **Human-readable sizes** — all byte counts are formatted with
  [`humansize`](https://crates.io/crates/humansize) (binary scale), so you
  see `3.6 GiB` instead of `3813 MB`.
- **Persisted settings** — refresh rate, active theme, and the flow toggle
  are saved to `settings.toml` (see [Configuration](#configuration)) and
  restored on next launch.
- **Switchable themes** — cycle between three palettes live with `t`:
  **mizu** (deep water), **abyss** (near-black), **coral** (warm reds).
- **Mizu palette** — extended deep-water theme: `MIZU_ABYSS` (#02081A),
  `MIZU_DEEP` (#04102A), `MIZU_WAVE`, `MIZU_RIPPLE`, `MIZU_BLUE`,
  `MIZU_TEAL`, `MIZU_CYAN`, `MIZU_FOAM`, `MIZU_SURF`, `MIZU_ACCENT`,
  `MIZU_WARM`, `MIZU_DIM`.

## Architecture

```
                ┌────────────────────┐
                │  metrics::collect  │   tokio task, 1 Hz
                │ sysinfo + GPU name │
                └─────────┬──────────┘
                          │ mpsc<Metrics>
                          ▼
 ┌────────────┐    ┌────────────────┐    ┌──────────────┐
 │ crossterm  │ →  │  App::tick     │ →  │ ui::render   │
 │  events    │    │ (drain + anim) │    │  (ratatui)   │
 └────────────┘    └────────────────┘    └──────────────┘
        ▲                 │                     │
        └──── tick loop ──┴─────────────────────┘
                (refresh_rate_ms from settings)
```

Files:

- `src/main.rs` — terminal setup, event/draw loop, key dispatch.
- `src/app.rs` — `App` state, channel drain, rolling history, theme/flow
  toggles.
- `src/metrics.rs` — async sysinfo collector + GPU name query + `Metrics`.
- `src/animation.rs` — `BadAppleState` frame ticker + RLE decoder.
- `src/bad_apple_frames.rs` — 6572 RLE-encoded frames (generated).
- `src/config/` — `Settings` (serde/toml) load/save + sanitizing.
- `src/ui/` — layout, `MizuTheme` struct + palettes, panels
  (cpu, ram, disk, net, wave/flow).

## Build

Requires Rust 1.74+ (edition 2021).

```bash
git clone https://github.com/kiy0w0/mizucli.git
cd mizucli
cargo build --release
./target/release/mizu
```

Or just:

```bash
cargo run --release
```

### Regenerating Bad Apple frames

`src/bad_apple_frames.rs` is checked in, but if you want to rebuild it
from a different source video:

1. Drop `video.mp4` into `src/data/` (this folder is `.gitignore`d).
2. `pip install opencv-python numpy`
3. `python extract_frames.py`

This re-emits `src/bad_apple_frames.rs` with fresh RLE frames at 80×45.

## Keys

| Key       | Action                              |
|-----------|-------------------------------------|
| `1`       | overview layout                     |
| `2`       | CPU-focused layout                  |
| `3`       | net/disk layout                     |
| `t`       | cycle theme (mizu → abyss → coral)  |
| `f`       | toggle Bad Apple flow on/off        |
| `q`       | quit                                |
| `Ctrl-C`  | force quit                          |

The active tab is highlighted in the footer, and the current theme name
shows next to the `t` hint.

## Configuration

Settings are stored as TOML and loaded at startup. The lookup path is:

1. `$MIZU_CONFIG_DIR/settings.toml` if the env var is set, else
2. `<user config dir>/mizu/settings.toml`
   (`%APPDATA%\mizu\settings.toml` on Windows, `~/.config/mizu` on Linux,
   `~/Library/Application Support/mizu` on macOS).

Example `settings.toml`:

```toml
refresh_rate_ms = 16
theme = "mizu"        # mizu | abyss | coral
flow_enabled = true
```

- `refresh_rate_ms` is clamped to `[16, 1000]`.
- A missing file or a parse error falls back to defaults — mizu never
  crashes on a bad config.
- `theme` and `flow_enabled` are written back automatically when you press
  `t` / `f`.

## Platforms

- **Windows 11** — primary target. GPU name uses PowerShell
  `Get-CimInstance Win32_VideoController` (the legacy `wmic` is removed in
  recent Windows builds).
- **Linux / macOS** — CPU, RAM, disk, network, and the Bad Apple flow all
  work; GPU name falls back to `N/A` (the CIM query is Windows-only).

Any UTF-8 terminal with a monospace font works — Windows Terminal,
Alacritty, WezTerm, Kitty, iTerm2.

## Dependencies

```toml
ratatui   = "0.28"
crossterm = "0.28"   # event-stream feature
sysinfo   = "0.32"
tokio     = "1"     # rt-multi-thread, macros, sync, time, signal
anyhow    = "1"
humansize = "2"     # human-readable byte formatting
dirs      = "6"     # config dir lookup
toml      = "0.8"   # settings serialization
serde     = "1"     # derive for Settings
```

No audio crates — earlier `rodio` attempts produced crackling on the
target machine and were removed.

## CI

`.github/workflows/ci.yml` runs on every push/PR to `main`, across
**Windows, Ubuntu, and macOS** × stable/beta Rust:

- `cargo build` / `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

## License

MIT © kiy0w0
