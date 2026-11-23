# lowend

A tiny, minimal bass practice app. First tool: a **tuner**.

- Dark, one screen, big readable note — built to glance at while you play.
- Standard 4-string tuning (E A D G), A4 = 440 Hz. 5-string / drop tunings later.

## Targets

| Target | Status |
|---|---|
| macOS desktop | first |
| Web app / PWA | planned |
| Windows desktop | planned |

## Run

```sh
cargo run --release          # dev
./scripts/bundle-macos.sh    # builds target/lowend.app (needed for mic permission)
cargo test
```

## Architecture

All Rust, one crate, split so porting touches one file:

```
src/pitch.rs   pitch detection (pYIN via `pyin`) + note/cents math. No I/O: runs on wasm as-is.
src/audio.rs   mic capture via `cpal`. The only platform-specific module.
src/i18n.rs    UI strings (IT, EN). Language follows the system, click the code to switch.
src/main.rs    UI with `egui`/`eframe` (runs native and in the browser).
```

Windows: same code, `cargo build`. Web/PWA: `eframe` on wasm + a Web Audio `audio.rs`.

## Design tokens

| Token | Value |
|---|---|
| `color/bg` | `#0B0B0C` |
| `color/line` | `#2A2A2E` |
| `color/text` | `#F2F2EF` |
| `color/muted` | `#6E6E74` |
| `color/ok` (in tune, ±5 ¢) | `#4ADE80` |
| `color/off` (out of tune) | `#F5A524` |

Fonts: [Geist / Geist Mono](https://vercel.com/font) (OFL, free for web and desktop).

## License

MIT
