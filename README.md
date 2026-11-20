# lowend

A tiny, minimal bass practice app. First tool: a **tuner**.

- Dark, one screen, big readable note — built to glance at while you play.
- Standard 4-string tuning (E A D G), A4 = 440 Hz. 5-string / drop tunings later.

**Design:** [Figma — Lowend, Bass Practice](https://www.figma.com/design/f5pwpIv8nYWVXfS50iiLQf)
(tokens, `String key` and `Cents meter` components, tuner states, PWA frame).

## Targets

| Target | Status |
|---|---|
| macOS desktop | first |
| Web app / PWA | planned |
| Windows desktop | planned |

## Architecture (portable by design)

```
core/      Rust crate: pitch detection (YIN), note/cents math. No audio, no UI deps.
           Builds native and to wasm32 — the same DSP runs everywhere.
web/       UI in plain HTML/CSS/TS. Mic via Web Audio (getUserMedia + AudioWorklet),
           calls core through wasm. This *is* the web app and the PWA.
desktop/   Tauri v2 shell around web/ — macOS now, Windows with the same code.
```

One DSP core, one UI, every platform. Lightweight: no framework, Tauri uses the system webview.

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
