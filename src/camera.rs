//! Orbital camera: azimuth, elevation, zoom, pan, and optional perspective.

use eframe::egui;
use std::f32::consts::FRAC_PI_2;

/// Maximum elevation (just below ±90° to avoid a degenerate view).
const MAX_ELEVATION: f32 = FRAC_PI_2 - 0.05;

/// Orbital camera controlled with the mouse.
///
/// Marked `#[non_exhaustive]`: start with `default()`.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct OrbitCamera {
    /// Azimuth (radians) around the vertical axis.
    pub orbit: f32,
    /// Elevation (radians): 0 = horizontal view, > 0 = seen from above.
    pub elevation: f32,
    /// Zoom factor.
    pub zoom: f32,
    /// Screen-space pan offset (pixels).
    pub pan_offset: egui::Vec2,
    /// `true`: perspective projection; `false`: orthographic.
    pub perspective: bool,
    /// Camera distance from the scene origin (world units, perspective only).
    pub distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            orbit: 0.5,
            elevation: 0.35,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            perspective: true,
            distance: 10.0,
        }
    }
}

impl OrbitCamera {
    /// Restores orbit, elevation, zoom, and pan to their default values
    /// (the projection mode is kept).
    pub fn reset(&mut self) {
        let perspective = self.perspective;
        *self = Self::default();
        self.perspective = perspective;
    }

    /// Applies mouse input: left-drag = orbit + tilt, scroll = zoom,
    /// right/middle-drag = pan.
    pub fn handle_input(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let delta = response.drag_delta();

        if response.dragged_by(egui::PointerButton::Primary) {
            self.orbit += delta.x * 0.01;
            self.elevation = (self.elevation + delta.y * 0.01).clamp(-MAX_ELEVATION, MAX_ELEVATION);
        }

        if response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle)
        {
            self.pan_offset += delta;
        }

        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom = (self.zoom * (scroll * 0.0015).exp()).clamp(0.1, 20.0);
            }
        }
    }
}