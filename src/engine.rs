//! High-level rendering engine: allocates the drawing area, handles the camera,
//! draws the ground/grid, then delegates scene drawing to the caller.

use crate::camera::OrbitCamera;
use crate::render::{Render3D, BASE_SCALE};
use eframe::egui;
/// Appearance options for the ground grid.
///
/// Marked `#[non_exhaustive]`: new fields may be added without breaking
/// compatibility. Outside the crate, start with `default()`, then use the
/// `with_*` methods or modify the fields:
///
/// ```ignore
/// let grid = GridStyle::default().with_step_px(30.0).with_half_lines(20);
/// ```
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct GridStyle {
    /// Grid-line color.
    pub line_color: egui::Color32,
    /// Ground-line color.
    pub ground_color: egui::Color32,
    /// Background color of the rendering area.
    pub background_color: egui::Color32,
    /// Grid-line spacing (pixels, before zoom).
    pub step_px: f32,
    /// Number of grid lines on each side of the center.
    pub half_lines: i32,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            line_color: egui::Color32::from_rgb(40, 50, 70),
            ground_color: egui::Color32::from_rgb(60, 70, 90),
            background_color: egui::Color32::from_rgb(25, 28, 36),
            step_px: 60.0,
            half_lines: 10,
        }
    }
}

impl GridStyle {
    /// Sets the grid-line color.
    #[must_use]
    pub fn with_line_color(mut self, color: egui::Color32) -> Self {
        self.line_color = color;
        self
    }

    /// Sets the ground-line color.
    #[must_use]
    pub fn with_ground_color(mut self, color: egui::Color32) -> Self {
        self.ground_color = color;
        self
    }

    /// Sets the background color.
    #[must_use]
    pub fn with_background_color(mut self, color: egui::Color32) -> Self {
        self.background_color = color;
        self
    }

    /// Sets the grid-line spacing (pixels, before zoom).
    #[must_use]
    pub fn with_step_px(mut self, step_px: f32) -> Self {
        self.step_px = step_px;
        self
    }

    /// Sets the number of grid lines on each side of the center.
    #[must_use]
    pub fn with_half_lines(mut self, half_lines: i32) -> Self {
        self.half_lines = half_lines;
        self
    }
}

/// 3D rendering engine: combines the orbital camera and scene rendering.
#[derive(Default)]
pub struct Engine3D {
    /// Orbital camera (angle, zoom, and pan).
    pub camera: OrbitCamera,
    /// Ground-grid appearance.
    pub grid: GridStyle,
}

impl Engine3D {
    /// Allocates the rendering area, updates the camera from user input, draws
    /// the background and the 3D ground grid, then calls `draw_callback` with
    /// a ready-to-use rendering context.
    pub fn render<F>(&mut self, ui: &mut egui::Ui, draw_callback: F)
    where
        F: FnOnce(&Render3D),
    {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());
        self.camera.handle_input(ui, &response);

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, self.grid.background_color);

        let center = rect.center() + self.camera.pan_offset;
        let render_ctx = Render3D {
            painter: &painter,
            center,
            ground_y: center.y + 150.0 * self.camera.zoom,
            camera_orbit: self.camera.orbit,
            camera_elevation: self.camera.elevation,
            zoom: self.camera.zoom,
            perspective: self.camera.perspective,
            camera_distance: self.camera.distance,
        };

        // Real 3D ground grid on the y = 0 plane (step_px is converted to world units).
        let step = self.grid.step_px / BASE_SCALE;
        let n = self.grid.half_lines;
        let ext = n as f32 * step;
        for i in -n..=n {
            let t = i as f32 * step;
            let stroke = if i == 0 {
                egui::Stroke::new(2.0, self.grid.ground_color)
            } else {
                egui::Stroke::new(1.0, self.grid.line_color)
            };
            render_ctx.draw_line([t, 0.0, -ext], [t, 0.0, ext], stroke);
            render_ctx.draw_line([-ext, 0.0, t], [ext, 0.0, t], stroke);
        }

        draw_callback(&render_ctx);
    }
}