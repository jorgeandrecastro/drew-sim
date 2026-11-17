//! Orbital camera and user input handling (zoom, pan, orbit).

use eframe::egui;

/// Default zoom bounds.
pub const DEFAULT_ZOOM_RANGE: (f32, f32) = (0.2, 5.0);

/// Orbital camera state: orbit angle, zoom, and pan offset.
#[derive(Clone, Copy, Debug)]
pub struct OrbitCamera {
    /// Horizontal orbit angle (radians) around the vertical axis.
    pub orbit: f32,
    /// Zoom factor, clamped to `zoom_range`.
    pub zoom: f32,
    /// Pan offset (screen pixels).
    pub pan_offset: egui::Vec2,
    /// Zoom bounds `(min, max)`.
    pub zoom_range: (f32, f32),
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            orbit: 0.5,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            zoom_range: DEFAULT_ZOOM_RANGE,
        }
    }
}

impl OrbitCamera {
    /// Resets the orbit, zoom, and pan to their default values
    /// (custom zoom bounds are preserved).
    pub fn reset(&mut self) {
        *self = Self { zoom_range: self.zoom_range, ..Self::default() };
    }

    /// Updates the camera from the egui interaction response for the rendering
    /// area (scroll wheel = zoom, right/middle-click and drag = pan,
    /// left-click and drag = orbit).
    pub fn handle_input(&mut self, ui: &egui::Ui, response: &egui::Response) {
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                self.zoom = (self.zoom + scroll_delta * 0.002)
                    .clamp(self.zoom_range.0, self.zoom_range.1);
            }
        }

        if response.dragged() {
            let delta = response.drag_delta();
            let panning = ui.input(|i| {
                i.pointer.button_down(egui::PointerButton::Secondary)
                    || i.pointer.button_down(egui::PointerButton::Middle)
            });
            if panning {
                self.pan_offset += delta;
            } else {
                self.orbit += delta.x * 0.01;
            }
        }
    }
}