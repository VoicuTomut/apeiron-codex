//! The blossom welcome animation for onboarding, with a full-color final pose.
//! Visible time pauses while hidden. Fresh conversations start settled and can replay on a click.

mod geometry;
mod lighting;
mod paths;
mod policy;
mod renderer;
mod sequence;

use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::MouseButton;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::motion::MotionMode;
use crate::terminal_palette;
use lighting::Lighting;
pub(crate) use policy::Presentation;
pub(crate) use policy::is_startup_cell;
use renderer::MAX_COLUMNS;
use renderer::MAX_ROWS;
use renderer::Renderer;

pub(crate) const FRAME_INTERVAL: Duration = Duration::from_millis(/*millis*/ 50);
// The preview's first visible pose (10.8s / 7.2s), also the start of an idle-screen replay.
const SETTLED_BLOSSOM: f64 = 0.5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComposerState {
    Empty,
    Draft,
}

#[derive(Default)]
pub(crate) struct EmptyStateAnimation {
    eligible: bool,
    spin_elapsed: Duration,
    last_frame: Option<Instant>,
    fade_elapsed: Duration,
    static_mark: Option<bool>,
    opacity: f32,
    fade_from: f32,
    renderer: Option<Renderer>,
    stage: Option<Rect>,
    replaying: bool,
}

/// The Apeiron Agents mark, as braille dot rows generated from the brand icon.
const APEIRON_MARK: &str = include_str!("../assets/apeiron_mark.txt");

/// Lay the mark out centered in `area`, returning one dot byte per cell in
/// row-major order. Cells the mark does not cover stay zero and are skipped by
/// the caller, so the stage keeps whatever the caller painted behind it.
fn apeiron_mark_dots(area: Rect) -> Vec<u8> {
    let rows: Vec<&str> = APEIRON_MARK.lines().collect();
    let mark_h = rows.len() as u16;
    let mark_w = rows
        .iter()
        .map(|row| row.chars().count() as u16)
        .max()
        .unwrap_or(0);

    let mut dots = vec![0u8; usize::from(area.width) * usize::from(area.height)];
    if mark_w == 0 || mark_h == 0 || area.width == 0 || area.height == 0 {
        return dots;
    }

    // Center, clipping rather than scaling when the stage is smaller than the mark.
    let x0 = area.width.saturating_sub(mark_w) / 2;
    let y0 = area.height.saturating_sub(mark_h) / 2;

    for (ry, row) in rows.iter().enumerate() {
        let y = y0 + ry as u16;
        if y >= area.height {
            break;
        }
        for (rx, ch) in row.chars().enumerate() {
            let x = x0 + rx as u16;
            if x >= area.width {
                break;
            }
            // Braille lives at U+2800 + dot bits; U+2800 itself is blank.
            let bits = (ch as u32).checked_sub(0x2800).unwrap_or(0);
            if bits == 0 || bits > 0xFF {
                continue;
            }
            dots[usize::from(y) * usize::from(area.width) + usize::from(x)] = bits as u8;
        }
    }
    dots
}

impl EmptyStateAnimation {
    pub(crate) fn is_eligible(&self) -> bool {
        self.eligible
    }

    pub(crate) fn start_fresh(&mut self) {
        self.cancel_replay();
        self.eligible = true;
        self.spin_elapsed = Duration::ZERO;
        self.last_frame = None;
        self.fade_elapsed = Duration::ZERO;
        self.static_mark = None;
        self.opacity = 1.0;
    }

    /// Keep the provisional pose when handing off to the live thread.
    pub(crate) fn continue_from(&mut self, source: &mut Self) {
        let mut previous = std::mem::take(source);
        if !previous.is_eligible() {
            previous.start_fresh();
        }
        *self = previous;
    }

    pub(crate) fn dismiss(&mut self) {
        self.eligible = false;
        self.cancel_replay();
    }

    pub(crate) fn cancel_replay(&mut self) {
        self.stage = None;
        self.replaying = false;
        self.pause_clock();
    }

    /// Replay only on the drawn blossom; unrelated gestures keep their normal owner.
    pub(crate) fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        let inside = self
            .stage
            .is_some_and(|stage| stage.contains(Position::new(mouse.column, mouse.row)))
            && mouse.modifiers.is_empty();
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) if inside => {
                self.start_fresh();
                self.spin_elapsed =
                    Duration::from_secs_f64(SETTLED_BLOSSOM * sequence::LOOP_SECONDS);
                self.opacity = sequence::STATIC_OPACITY;
                self.static_mark = Some(true);
                self.replaying = true;
                true
            }
            _ => false,
        }
    }

    /// Stop visible time at the last painted frame, including when no hidden frame was drawn.
    /// Resume (after process suspension) may call this after the interruption; phase is retained.
    pub(crate) fn pause_clock(&mut self) {
        self.last_frame = None;
    }

    /// Paint the shared logo sequence inside a caller-owned, reserved rectangle.
    /// The caller clears the stage and keeps its layout stable after motion finishes.
    pub(crate) fn render_in(
        &mut self,
        area: Rect,
        buffer: &mut Buffer,
        presentation: Presentation,
    ) -> Option<Duration> {
        let now = Instant::now();
        self.render_in_at(area, buffer, presentation, now)
    }

    fn render_in_at(
        &mut self,
        area: Rect,
        buffer: &mut Buffer,
        presentation: Presentation,
        now: Instant,
    ) -> Option<Duration> {
        if !self.eligible
            || presentation == Presentation::Hidden
            || area.is_empty()
            || area.width > MAX_COLUMNS
            || area.height > MAX_ROWS
            || area.intersection(buffer.area) != area
        {
            self.pause_clock();
            return None;
        }
        let static_mark = presentation == Presentation::Faded;
        let previous_frame = self.last_frame.replace(now);
        if self.static_mark != Some(static_mark) {
            self.fade_elapsed = if self.static_mark.is_none() {
                sequence::STATIC_FADE
            } else {
                Duration::ZERO
            };
            self.fade_from = self.opacity;
        } else if let Some(previous) = previous_frame {
            let elapsed = now.saturating_duration_since(previous);
            self.fade_elapsed += elapsed;
            if !static_mark {
                self.spin_elapsed += elapsed;
            }
        }
        self.static_mark = Some(static_mark);
        let finished = self.spin_elapsed >= sequence::SPIN_DURATION;
        let phase =
            self.spin_elapsed.min(sequence::SPIN_DURATION).as_secs_f64() / sequence::LOOP_SECONDS;
        let settling = !finished && static_mark && self.fade_elapsed < sequence::STATIC_FADE;
        self.opacity = if finished && self.replaying {
            sequence::static_opacity(
                self.spin_elapsed - sequence::SPIN_DURATION,
                /*from*/ 1.0,
            )
        } else if finished {
            1.0
        } else if static_mark {
            sequence::static_opacity(self.fade_elapsed, self.fade_from)
        } else {
            1.0
        };
        if !static_mark && !finished {
            self.opacity = self.fade_from
                + (self.opacity - self.fade_from)
                    * sequence::progress(self.fade_elapsed, sequence::STATIC_FADE) as f32;
        }
        self.paint_frame(area, buffer, phase, self.opacity);
        ((!finished && (!static_mark || settling))
            || (self.replaying
                && self.spin_elapsed < sequence::SPIN_DURATION + sequence::STATIC_FADE))
            .then_some(FRAME_INTERVAL)
    }

    /// Draw only in space the caller has cleared and owns. The centered blossom returns
    /// on clear, without restarting or scheduling animation.
    pub(crate) fn render_first_screen(
        &mut self,
        available: Rect,
        buffer: &mut Buffer,
        composer: Option<ComposerState>,
        motion: MotionMode,
    ) -> Option<Duration> {
        const STAGE_ROWS: u16 = 21;
        const MIN_STAGE_ROWS: u16 = 14;
        let screen = buffer.area;
        let width = screen.width.saturating_sub(/*rhs*/ 4).min(MAX_COLUMNS);
        let height = width * STAGE_ROWS / MAX_COLUMNS;
        if motion == MotionMode::Reduced
            || composer != Some(ComposerState::Empty)
            || height < MIN_STAGE_ROWS
            || height > screen.height
        {
            self.cancel_replay();
            return None;
        }
        let stage = Rect::new(
            screen.x + (screen.width - width) / 2,
            screen.y + (screen.height - height) / 2,
            width,
            height,
        );
        if stage.intersection(available) != stage {
            self.cancel_replay();
            return None;
        }
        self.stage = Some(stage);
        if self.replaying {
            if let Some(delay) = self.render_in(stage, buffer, Presentation::Animated) {
                return Some(delay);
            }
            // The replay has faded back to the idle pose and no longer needs redraws.
            self.replaying = false;
        }
        self.paint_frame(stage, buffer, SETTLED_BLOSSOM, sequence::STATIC_OPACITY);
        None
    }

    fn paint_frame(&mut self, area: Rect, buffer: &mut Buffer, phase: f64, opacity: f32) {
        let background = terminal_palette::default_bg();
        let color_level = if background.is_some() {
            terminal_palette::effective_stdout_color_level()
        } else {
            terminal_palette::StdoutColorLevel::Unknown
        };
        let background = background.unwrap_or((15, 20, 37));
        let light = Lighting::terminal(
            terminal_palette::default_fg().unwrap_or((210, 221, 235)),
            background,
        );
        // Apeiron Agents paints its own mark in place of upstream's procedural
        // blossom. The renderer still runs so the lighting model keeps supplying
        // per-cell shading, but the dot pattern comes from the brand mark and the
        // hue is the Apeiron accent. The fade and scheduling above are untouched,
        // so the mark still fades in and callers' sizing still applies.
        let shading = self.renderer.get_or_insert_with(Renderer::default).frame(
            area.width,
            area.height,
            phase,
            &light,
        );
        let mark = apeiron_mark_dots(area);
        let (ar, ag, ab) = crate::style::APEIRON_GREEN_200;
        for (i, dots) in mark.iter().copied().enumerate().filter(|(_, dots)| *dots != 0) {
            // Shade the brand hue with the renderer's local luminance so the mark
            // keeps the depth the animation gives it rather than going flat.
            let shade = shading
                .get(i)
                .map(|cell| {
                    let [_, r, g, b] = cell.rgb.to_be_bytes();
                    (u16::from(r) + u16::from(g) + u16::from(b)) / 3
                })
                .unwrap_or(255)
                .max(96);
            let scale = f32::from(shade) / 255.0;
            let (r, g, b) = (
                (f32::from(ar) * scale) as u8,
                (f32::from(ag) * scale) as u8,
                (f32::from(ab) * scale) as u8,
            );
            let target = &mut buffer[(
                area.x + i as u16 % area.width,
                area.y + i as u16 / area.width,
            )];
            target.set_char(char::from_u32(0x2800 + u32::from(dots)).unwrap_or(' '));
            let color = crate::color::blend((r, g, b), background, opacity);
            let color = if color_level == terminal_palette::StdoutColorLevel::Ansi256 {
                // Four bits per channel bound the cache and avoid palette searches each frame.
                static COLORS: [OnceLock<Color>; 4096] = [const { OnceLock::new() }; 4096];
                let (r, g, b) = (color.0 >> 4, color.1 >> 4, color.2 >> 4);
                let index = usize::from(r) * 256 + usize::from(g) * 16 + usize::from(b);
                *COLORS[index].get_or_init(|| {
                    terminal_palette::best_color_for_level(
                        (r * 16 + 8, g * 16 + 8, b * 16 + 8),
                        color_level,
                    )
                })
            } else {
                terminal_palette::best_color_for_level(color, color_level)
            };
            target.set_fg(color);
            // Default-color terminals can only step between normal and dim intensity.
            if opacity < 0.5
                && matches!(
                    color_level,
                    terminal_palette::StdoutColorLevel::Ansi16
                        | terminal_palette::StdoutColorLevel::Unknown
                )
            {
                target.set_style(target.style().dim());
            }
        }
    }
}

#[cfg(test)]
#[path = "empty_state_animation_tests.rs"]
mod tests;
