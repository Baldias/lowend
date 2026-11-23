mod audio;
mod i18n;
mod pitch;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Sense, Stroke, StrokeKind,
    Vec2, pos2, text::LayoutJob, vec2,
};
use i18n::{Lang, Msg};
use pitch::Note;
use std::sync::Arc;

// Design tokens (see README).
const BG: Color32 = Color32::from_rgb(0x0B, 0x0B, 0x0C);
const LINE: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x2E);
const TEXT: Color32 = Color32::from_rgb(0xF2, 0xF2, 0xEF);
const MUTED: Color32 = Color32::from_rgb(0x6E, 0x6E, 0x74);
const OK: Color32 = Color32::from_rgb(0x4A, 0xDE, 0x80);
const OFF: Color32 = Color32::from_rgb(0xF5, 0xA5, 0x24);

const A4: f32 = 440.0;
const IN_TUNE_CENTS: f32 = 5.0;
/// Standard 4-string tuning as MIDI numbers: E1 A1 D2 G2.
const STRINGS: [i32; 4] = [28, 33, 38, 43];
/// Keep showing the last reading this long after the signal drops (seconds).
const HOLD: f64 = 0.6;

struct Reading {
    hz: f32,
    seen_at: f64,
}

struct App {
    mic: Result<(audio::Mic, pitch::Detector), String>,
    lang: Lang,
    reading: Option<Reading>,
    /// Needle position actually drawn, eased toward the measured cents every frame.
    needle: f32,
    last_detect: f64,
}

/// Pitch analyses per second; the needle animates at full frame rate in between.
const DETECT_HZ: f64 = 25.0;
/// Needle easing speed (1/s): higher = snappier, lower = smoother.
const NEEDLE_SPEED: f32 = 14.0;

impl App {
    fn update_reading(&mut self, now: f64) {
        if now - self.last_detect < 1.0 / DETECT_HZ {
            return;
        }
        self.last_detect = now;
        let Ok((mic, detector)) = &mut self.mic else { return };
        if let Some(hz) = mic.window().and_then(|w| detector.detect(&w)) {
            self.reading = Some(match &self.reading {
                // Smooth in log space so the needle doesn't jitter; jump on a new note.
                Some(r) if (1200.0 * (hz / r.hz).log2()).abs() < 50.0 => {
                    Reading { hz: r.hz * (hz / r.hz).powf(0.5), seen_at: now }
                }
                _ => Reading { hz, seen_at: now },
            });
        } else if self.reading.as_ref().is_some_and(|r| now - r.seen_at > HOLD) {
            self.reading = None;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let (now, dt) = ui.input(|i| (i.time, i.stable_dt));
        self.update_reading(now);
        ui.ctx().request_repaint();

        let r = ui.max_rect();
        let p = ui.painter();
        p.rect_filled(r, 0.0, BG);
        let mono = |s| FontId::new(s, FontFamily::Monospace);
        let cx = r.center().x;

        // Header: tuning on the right, language toggle at the far right.
        let hy = r.top() + 24.0;
        let lang_rect = text(p, pos2(r.right() - 24.0, hy), Align2::RIGHT_CENTER, self.lang.code(), mono(11.0), MUTED, 0.4);
        let lang = ui.interact(lang_rect.expand(6.0), ui.scope_id().with("lang"), Sense::click());
        if lang.hovered() {
            text(p, pos2(r.right() - 24.0, hy), Align2::RIGHT_CENTER, self.lang.code(), mono(11.0), TEXT, 0.4);
        }
        if lang.clicked() {
            self.lang = self.lang.next();
        }
        let tuning = format!("{} · A4 {}", self.lang.t(Msg::Standard), A4);
        text(p, pos2(lang_rect.left() - 16.0, hy), Align2::RIGHT_CENTER, &tuning, mono(11.0), MUTED, 0.4);

        let note = self.reading.as_ref().map(|rd| (Note::from_hz(rd.hz, A4), rd.hz));
        let in_tune = note.is_some_and(|(n, _)| n.cents.abs() <= IN_TUNE_CENTS);
        let accent = if in_tune { OK } else { OFF };
        if let Some((n, _)) = note {
            self.needle += (n.cents - self.needle) * (1.0 - (-dt * NEEDLE_SPEED).exp());
        }

        // Bottom-up layout: string keys, then meter, readout centered in the remaining space.
        let keys_top = r.bottom() - 24.0 - 56.0;
        let meter_top = keys_top - 40.0 - 72.0;
        let ry = (r.top() + 40.0 + meter_top) / 2.0;

        // Readout
        let big = FontId::new(168.0, FontFamily::Name("semibold".into()));
        let (label, color) = match note {
            Some((n, _)) => (n.name(), if in_tune { OK } else { TEXT }),
            // Invisible "E" keeps the layout; the idle dot is drawn as a shape (no font glyph needed).
            None => ("E", Color32::TRANSPARENT),
        };
        let nr = text(p, pos2(cx, ry - 20.0), Align2::CENTER_CENTER, label, big, color, -6.0);
        if note.is_none() {
            p.circle_filled(pos2(cx, nr.center().y + 10.0), 8.0, LINE);
        }
        let sub = match note {
            Some((n, hz)) => {
                text(p, pos2(nr.right() + 4.0, nr.bottom() - 40.0), Align2::LEFT_BOTTOM, &n.octave().to_string(), mono(24.0), MUTED, 0.0);
                let mut job = LayoutJob::default();
                job.append(&format!("{:+.0} ¢", n.cents), 0.0, fmt(mono(14.0), accent, 0.0));
                job.append(&format!("{hz:.2} Hz"), 16.0, fmt(mono(14.0), MUTED, 0.0));
                job
            }
            None => {
                let msg = if self.mic.is_ok() { Msg::PlayAString } else { Msg::NoMic };
                LayoutJob::simple_singleline(self.lang.t(msg).into(), FontId::proportional(14.0), MUTED)
            }
        };
        let g = p.layout_job(sub);
        p.galley(Align2::CENTER_TOP.anchor_size(pos2(cx, nr.bottom() - 16.0), g.size()).min, g, MUTED);

        let needle = note.map(|_| self.needle);
        meter(p, Rect::from_center_size(pos2(cx, meter_top + 36.0), vec2(312.0, 72.0)), needle, accent);

        // String keys
        let size = 56.0;
        let gap = 12.0;
        let x0 = cx - (STRINGS.len() as f32 * (size + gap) - gap) / 2.0;
        for (i, &midi) in STRINGS.iter().enumerate() {
            let k = Rect::from_min_size(pos2(x0 + i as f32 * (size + gap), keys_top), Vec2::splat(size));
            let hit = note.is_some_and(|(n, _)| n.midi == midi);
            let (fill, stroke, tc) = match (hit, in_tune) {
                (true, true) => (OK, Stroke::NONE, BG),
                (true, false) => (Color32::TRANSPARENT, Stroke::new(1.5, OFF), TEXT),
                _ => (Color32::TRANSPARENT, Stroke::new(1.5, LINE), MUTED),
            };
            p.rect(k, CornerRadius::same(14), fill, stroke, StrokeKind::Inside);
            let name = Note { midi, cents: 0.0 }.name();
            text(p, k.center(), Align2::CENTER_CENTER, name, FontId::new(18.0, FontFamily::Name("medium".into())), tc, 0.0);
        }
    }
}

/// Cents meter: ticks every 5 ¢ from −50 to +50, green zone ±5 ¢, needle at `cents`.
fn meter(p: &egui::Painter, r: Rect, cents: Option<f32>, accent: Color32) {
    let span = 144.0;
    let x = |c: f32| r.center().x + c.clamp(-50.0, 50.0) / 50.0 * span;
    let ticks_y = r.top() + 20.0;
    let zone = Rect::from_x_y_ranges(x(-IN_TUNE_CENTS)..=x(IN_TUNE_CENTS), r.top() + 4.0..=r.top() + 36.0);
    let zone_alpha = if cents.is_some_and(|c| c.abs() <= IN_TUNE_CENTS) { 0.18 } else { 0.08 };
    p.rect_filled(zone, 4.0, OK.gamma_multiply(zone_alpha));
    for k in (-50..=50).step_by(5) {
        let (h, w, c) = match k {
            0 => (32.0, 2.0, TEXT),
            _ if k % 25 == 0 => (20.0, 1.0, LINE),
            _ => (12.0, 1.0, LINE),
        };
        let c = if cents.is_none() { c.gamma_multiply(0.6) } else { c };
        p.rect_filled(Rect::from_center_size(pos2(x(k as f32), ticks_y), vec2(w, h)), 0.0, c);
    }
    if let Some(c) = cents {
        p.rect_filled(Rect::from_center_size(pos2(x(c), ticks_y), vec2(4.0, 44.0)), 2.0, accent);
    }
    for (label, k) in [("−50", -50.0), ("0", 0.0), ("+50", 50.0)] {
        text(p, pos2(x(k), r.bottom() - 4.0), Align2::CENTER_BOTTOM, label, FontId::monospace(11.0), MUTED, 0.4);
    }
}

fn fmt(font_id: FontId, color: Color32, extra_letter_spacing: f32) -> egui::TextFormat {
    egui::TextFormat { font_id, color, extra_letter_spacing, ..Default::default() }
}

/// Paints one line of text anchored at `pos`, returns its rect.
fn text(p: &egui::Painter, pos: Pos2, align: Align2, s: &str, font: FontId, color: Color32, spacing: f32) -> Rect {
    let mut job = LayoutJob::default();
    job.append(s, 0.0, fmt(font, color, spacing));
    let g = p.layout_job(job);
    let rect = align.anchor_size(pos, g.size());
    p.galley(rect.min, g, color);
    rect
}

fn fonts() -> egui::FontDefinitions {
    let mut f = egui::FontDefinitions::default();
    for (name, bytes) in [
        ("regular", include_bytes!("../assets/fonts/Geist-Regular.ttf").as_slice()),
        ("medium", include_bytes!("../assets/fonts/Geist-Medium.ttf").as_slice()),
        ("semibold", include_bytes!("../assets/fonts/Geist-SemiBold.ttf").as_slice()),
        ("mono", include_bytes!("../assets/fonts/GeistMono-Regular.ttf").as_slice()),
    ] {
        f.font_data.insert(name.into(), Arc::new(egui::FontData::from_static(bytes)));
    }
    // Default fonts stay as fallback for glyphs Geist lacks.
    f.families.entry(FontFamily::Proportional).or_default().insert(0, "regular".into());
    f.families.entry(FontFamily::Monospace).or_default().insert(0, "mono".into());
    let fallback = f.families[&FontFamily::Proportional].clone();
    for n in ["medium", "semibold"] {
        f.families.insert(FontFamily::Name(n.into()), [vec![n.to_string()], fallback.clone()].concat());
    }
    f
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("lowend")
            .with_inner_size([360.0, 480.0])
            .with_min_inner_size([340.0, 440.0])
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            .with_movable_by_background(true),
        ..Default::default()
    };
    eframe::run_native(
        "lowend",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_fonts(fonts());
            let mic = audio::Mic::open().map(|m| {
                let d = pitch::Detector::new(m.sample_rate);
                (m, d)
            });
            Ok(Box::new(App { mic, lang: Lang::detect(), reading: None, needle: 0.0, last_detect: 0.0 }))
        }),
    )
}
