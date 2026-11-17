//! Rendering context: 3D-to-2D projection and drawing primitives.

use crate::math::{add, cross, dot, mat3_mul_vec, normalize, scale, sub, Rotation3D, Vec3};
use crate::mesh::Mesh3D;
use eframe::egui;

/// Pixels per world unit at zoom 1.0 (at depth 0).
pub const BASE_SCALE: f32 = 50.0;
/// Near-plane distance in front of the camera (perspective).
const NEAR: f32 = 0.1;
/// Fixed light direction in view space (camera looks toward +Z).
const LIGHT: Vec3 = [-0.35, 0.45, -0.82];

/// Context passed to the drawing callback for a frame.
pub struct Render3D<'a> {
    /// egui painter clipped to the rendering area.
    pub painter: &'a egui::Painter,
    /// Scene center on screen (including pan).
    pub center: egui::Pos2,
    /// Screen-space vertical coordinate of the world origin.
    pub ground_y: f32,
    /// Camera azimuth (radians) around the vertical axis.
    pub camera_orbit: f32,
    /// Camera elevation (radians).
    pub camera_elevation: f32,
    /// Zoom factor.
    pub zoom: f32,
    /// Perspective (`true`) or orthographic (`false`) projection.
    pub perspective: bool,
    /// Camera distance from the origin (world units, perspective only).
    pub camera_distance: f32,
}

fn shade(c: egui::Color32, k: f32) -> egui::Color32 {
    let f = |v: u8| (f32::from(v) * k).clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

impl Render3D<'_> {
    /// World point -> view space: x right, y up, z = depth (away from the camera).
    #[must_use]
    pub fn world_to_view(&self, pos: Vec3) -> Vec3 {
        let (so, co) = self.camera_orbit.sin_cos();
        let (se, ce) = self.camera_elevation.sin_cos();
        let x1 = pos[0] * co - pos[2] * so;
        let z1 = pos[0] * so + pos[2] * co;
        [x1, pos[1] * ce + z1 * se, z1 * ce - pos[1] * se]
    }

    /// Projects a view-space point; `None` if it is behind the camera.
    #[must_use]
    pub fn project_view(&self, v: Vec3) -> Option<egui::Pos2> {
        let f = if self.perspective {
            let d = self.camera_distance + v[2];
            if d < NEAR * 0.5 {
                return None;
            }
            self.camera_distance / d
        } else {
            1.0
        };
        let s = BASE_SCALE * self.zoom * f;
        Some(egui::pos2(self.center.x + v[0] * s, self.ground_y - v[1] * s))
    }

    /// Projects a world point to the screen.
    ///
    /// Points behind the camera (perspective) are clamped to the near plane;
    /// primitives that need exact clipping use [`Self::project_view`].
    #[must_use]
    pub fn world_to_screen(&self, pos: Vec3) -> egui::Pos2 {
        let mut v = self.world_to_view(pos);
        if self.perspective {
            v[2] = v[2].max(NEAR - self.camera_distance);
        }
        self.project_view(v).unwrap_or(self.center)
    }

    /// Clips a view-space segment against the near plane.
    fn clip_near(&self, a: Vec3, b: Vec3) -> Option<(Vec3, Vec3)> {
        if !self.perspective {
            return Some((a, b));
        }
        let near = NEAR - self.camera_distance;
        match (a[2] >= near, b[2] >= near) {
            (true, true) => Some((a, b)),
            (false, false) => None,
            (ina, _) => {
                let t = (near - a[2]) / (b[2] - a[2]);
                let p = add(a, scale(sub(b, a), t));
                if ina { Some((a, p)) } else { Some((p, b)) }
            }
        }
    }

    fn segment_view(&self, a: Vec3, b: Vec3, stroke: egui::Stroke) -> Option<egui::Shape> {
        let (a, b) = self.clip_near(a, b)?;
        Some(egui::Shape::line_segment([self.project_view(a)?, self.project_view(b)?], stroke))
    }

    /// Draws a 3D line segment between two world points.
    pub fn draw_line(&self, a: Vec3, b: Vec3, stroke: egui::Stroke) {
        if let Some(s) = self.segment_view(self.world_to_view(a), self.world_to_view(b), stroke) {
            self.painter.add(s);
        }
    }

    /// Draws a wireframe mesh at the given position and orientation.
    pub fn draw_mesh(&self, mesh: &Mesh3D, position: Vec3, rotation: Rotation3D, color: egui::Color32) {
        self.draw_mesh_with_stroke(mesh, position, rotation, egui::Stroke::new(1.0, color));
    }

    /// Variant of `draw_mesh` with a complete `Stroke` (including line width).
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
        let view: Vec<Vec3> = mesh
            .vertices
            .iter()
            .map(|&v| self.world_to_view(add(mat3_mul_vec(&m, v), position)))
            .collect();

        let shapes: Vec<egui::Shape> = mesh
            .edges()
            .iter()
            .filter_map(|&[a, b]| self.segment_view(*view.get(a)?, *view.get(b)?, stroke))
            .collect();
        self.painter.extend(shapes);
    }

    /// Draws a mesh with flat-shaded solid faces (painter's algorithm:
    /// triangles sorted back to front, two-sided lighting).
    ///
    /// Without a z-buffer, strongly concave meshes or interpenetrating
    /// objects can show sorting artifacts.
    pub fn draw_mesh_solid(&self, mesh: &Mesh3D, position: Vec3, rotation: Rotation3D, color: egui::Color32) {
        if mesh.vertices.is_empty() {
            return;
        }
        let m = rotation.to_matrix();
        let view: Vec<Vec3> = mesh
            .vertices
            .iter()
            .map(|&v| self.world_to_view(add(mat3_mul_vec(&m, v), position)))
            .collect();

        let mut faces: Vec<(f32, [usize; 3])> = Vec::with_capacity(mesh.triangles.len());
        for t in &mesh.triangles {
            let idx = [t[0] as usize, t[1] as usize, t[2] as usize];
            if idx.iter().any(|&i| i >= view.len()) {
                continue;
            }
            let depth = (view[idx[0]][2] + view[idx[1]][2] + view[idx[2]][2]) / 3.0;
            faces.push((depth, idx));
        }
        faces.sort_by(|a, b| b.0.total_cmp(&a.0)); // far -> near

        let mut out = egui::Mesh::default();
        for (_, [a, b, c]) in faces {
            let (va, vb, vc) = (view[a], view[b], view[c]);
            let (Some(pa), Some(pb), Some(pc)) =
                (self.project_view(va), self.project_view(vb), self.project_view(vc))
            else {
                continue;
            };
            let n = normalize(cross(sub(vb, va), sub(vc, va)));
            let col = shade(color, 0.25 + 0.75 * dot(n, LIGHT).abs());
            let i = out.vertices.len() as u32;
            out.colored_vertex(pa, col);
            out.colored_vertex(pb, col);
            out.colored_vertex(pc, col);
            out.add_triangle(i, i + 1, i + 2);
        }
        self.painter.add(egui::Shape::mesh(out));
    }

    /// Draws the XYZ axes (red = X, green = Y, blue = Z) at a position.
    pub fn draw_axes(&self, position: Vec3, rotation: Rotation3D, length: f32) {
        let m = rotation.to_matrix();
        let tip = |axis: Vec3| add(position, mat3_mul_vec(&m, axis));
        self.draw_line(position, tip([length, 0.0, 0.0]), egui::Stroke::new(2.5, egui::Color32::RED));
        self.draw_line(position, tip([0.0, length, 0.0]), egui::Stroke::new(2.5, egui::Color32::GREEN));
        self.draw_line(position, tip([0.0, 0.0, length]), egui::Stroke::new(2.5, egui::Color32::BLUE));
    }

    /// Draws a trail history (polyline; parts behind the camera are cut).
    pub fn draw_trail(&self, points: &[Vec3], color: egui::Color32) {
        if points.len() < 2 {
            return;
        }
        let stroke = egui::Stroke::new(1.5, color);
        let near = NEAR - self.camera_distance;
        let mut run: Vec<egui::Pos2> = Vec::new();
        let mut shapes: Vec<egui::Shape> = Vec::new();

        for &p in points {
            let v = self.world_to_view(p);
            let visible = !self.perspective || v[2] >= near;
            match (visible, self.project_view(v)) {
                (true, Some(s)) => run.push(s),
                _ => {
                    if run.len() >= 2 {
                        shapes.push(egui::Shape::line(std::mem::take(&mut run), stroke));
                    }
                    run.clear();
                }
            }
        }
        if run.len() >= 2 {
            shapes.push(egui::Shape::line(run, stroke));
        }
        self.painter.extend(shapes);
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