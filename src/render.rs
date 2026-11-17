//! Rendering context: 3D-to-2D projection and drawing primitives.

use crate::math::{add, mat3_mul_vec, Rotation3D, Vec3};
use crate::mesh::Mesh3D;
use eframe::egui;

/// Context passed to the drawing callback for a frame.
///
/// Provides everything needed to project a 3D point onto the screen and draw
/// on top of it (meshes, axes, trails, and charts).
pub struct Render3D<'a> {
    /// egui painter clipped to the rendering area.
    pub painter: &'a egui::Painter,
    /// Scene center on screen (including pan).
    pub center: egui::Pos2,
    /// Screen-space vertical coordinate of the ground (y = 0).
    pub ground_y: f32,
    /// Camera orbit angle (radians) around the vertical axis.
    pub camera_orbit: f32,
    /// Zoom factor.
    pub zoom: f32,
}

impl Render3D<'_> {
    /// Projects a point in the 3D world to a 2D screen point.
    ///
    /// Simplified orthographic projection with orbital rotation around the
    /// vertical axis; there is no perspective or depth, and no point can be
    /// "behind" the camera.
    #[must_use]
    pub fn world_to_screen(&self, pos: Vec3) -> egui::Pos2 {
        let base_scale = 50.0 * self.zoom;
        let (sin_o, cos_o) = self.camera_orbit.sin_cos();
        let rot_x = pos[0] * cos_o - pos[2] * sin_o;

        egui::pos2(
            self.center.x + (rot_x * base_scale),
            self.ground_y - (pos[1] * base_scale),
        )
    }

    /// Draws a wireframe mesh at the given position and orientation.
    pub fn draw_mesh(&self, mesh: &Mesh3D, position: Vec3, rotation: Rotation3D, color: egui::Color32) {
        self.draw_mesh_with_stroke(mesh, position, rotation, egui::Stroke::new(1.0, color));
    }

    /// Variant of `draw_mesh` with a complete `Stroke` (including line width).
    ///
    /// Each vertex is rotated and projected only once, and each shared edge is
    /// drawn only once (see [`Mesh3D::edges`]).
    pub fn draw_mesh_with_stroke(
        &self,
        mesh: &Mesh3D,
        position: Vec3,
        rotation: Rotation3D,
        stroke: egui::Stroke,
    ) {
        if mesh.vertices.is_empty() {
            return;
        }

        let m = rotation.to_matrix();
        let screen: Vec<egui::Pos2> = mesh
            .vertices
            .iter()
            .map(|&v| self.world_to_screen(add(mat3_mul_vec(&m, v), position)))
            .collect();

        let shapes: Vec<egui::Shape> = mesh
            .edges()
            .iter()
            .filter_map(|&[a, b]| {
                Some(egui::Shape::line_segment([*screen.get(a)?, *screen.get(b)?], stroke))
            })
            .collect();

        self.painter.extend(shapes);
    }

    /// Draws the XYZ axes (red = X, green = Y, blue = Z) at a position.
    pub fn draw_axes(&self, position: Vec3, rotation: Rotation3D, length: f32) {
        let m = rotation.to_matrix();
        let tip = |axis: Vec3| self.world_to_screen(add(position, mat3_mul_vec(&m, axis)));

        let p_origin = self.world_to_screen(position);
        let p_x = tip([length, 0.0, 0.0]);
        let p_y = tip([0.0, length, 0.0]);
        let p_z = tip([0.0, 0.0, length]);

        self.painter.line_segment([p_origin, p_x], egui::Stroke::new(2.5, egui::Color32::RED));
        self.painter.line_segment([p_origin, p_y], egui::Stroke::new(2.5, egui::Color32::GREEN));
        self.painter.line_segment([p_origin, p_z], egui::Stroke::new(2.5, egui::Color32::BLUE));
    }

    /// Draws a trail history (a polyline connecting `points`).
    pub fn draw_trail(&self, points: &[Vec3], color: egui::Color32) {
        if points.len() < 2 {
            return;
        }
        let pts: Vec<egui::Pos2> = points.iter().map(|&p| self.world_to_screen(p)).collect();
        self.painter.add(egui::Shape::line(pts, egui::Stroke::new(1.5, color)));
    }

    /// Draws a 2D chart (value curve) in the given screen rectangle,
    /// independent of the 3D camera.
    pub fn draw_chart(&self, rect: egui::Rect, values: &[f32], min_val: f32, max_val: f32, color: egui::Color32) {
        if values.len() < 2 || (max_val - min_val).abs() < f32::EPSILON {
            return;
        }

        self.painter.rect_filled(rect, 4.0, egui::Color32::from_black_alpha(150));
        self.painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, egui::Color32::GRAY),
            egui::StrokeKind::Middle,
        );

        let (width, height, len) = (rect.width(), rect.height(), values.len());
        let pts: Vec<egui::Pos2> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x_norm = i as f32 / (len - 1) as f32;
                let y_norm = ((v - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
                egui::pos2(rect.min.x + x_norm * width, rect.max.y - y_norm * height)
            })
            .collect();

        self.painter.add(egui::Shape::line(pts, egui::Stroke::new(1.5, color)));
    }
}