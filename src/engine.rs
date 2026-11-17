//! High-level rendering engine: allocates the drawing area, handles the camera,
//! draws the ground/grid, then delegates scene drawing to the caller.

use crate::camera::OrbitCamera;
use crate::render::Render3D;
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
    /// the background/grid, then calls `draw_callback` with a ready-to-use
    /// rendering context for drawing the scene.
    pub fn render<F>(&mut self, ui: &mut egui::Ui, draw_callback: F)
    where
        F: FnOnce(&Render3D),
    {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());
        self.camera.handle_input(ui, &response);

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, self.grid.background_color);

        let center = rect.center() + self.camera.pan_offset;
        let ground_y = center.y + (150.0 * self.camera.zoom);
        let grid_step = self.grid.step_px * self.camera.zoom;

        painter.line_segment(
            [egui::pos2(rect.min.x, ground_y), egui::pos2(rect.max.x, ground_y)],
            egui::Stroke::new(2.0, self.grid.ground_color),
        );

        for i in -self.grid.half_lines..=self.grid.half_lines {
            let offset = i as f32 * grid_step + (self.camera.orbit.sin() * 40.0 * self.camera.zoom);
            let start_x = center.x + offset;
            if start_x >= rect.min.x && start_x <= rect.max.x {
                painter.line_segment(
                    [egui::pos2(start_x, ground_y), egui::pos2(start_x + (offset * 0.5), rect.max.y)],
                    egui::Stroke::new(1.0, self.grid.line_color),
                );
            }
        }

        let render_ctx = Render3D {
            painter: &painter,
            center,
            ground_y,
            camera_orbit: self.camera.orbit,
            zoom: self.camera.zoom,
        };

        draw_callback(&render_ctx);
    }
}