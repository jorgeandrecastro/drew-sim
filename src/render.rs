//! Contexte de rendu : projection 3D -> 2D et primitives de dessin.

use crate::math::{Rotation3D, Vec3};
use crate::mesh::Mesh3D;
use eframe::egui;

/// Contexte transmis lors du callback de dessin d'une frame.
///
/// Regroupe tout ce qu'il faut pour projeter un point 3D vers l'écran et
/// dessiner par-dessus (maillages, repères, trajectoires, graphes).
pub struct Render3D<'a> {
    pub painter: &'a egui::Painter,
    pub center: egui::Pos2,
    pub ground_y: f32,
    pub camera_orbit: f32,
    pub zoom: f32,
}

impl<'a> Render3D<'a> {
    /// Projection d'un point du monde 3D vers un point écran 2D.
    ///
    /// Projection orthographique simplifiée avec rotation orbitale sur
    /// l'axe horizontal ; suffisante pour de la télémétrie temps réel.
    pub fn world_to_screen(&self, pos: Vec3) -> egui::Pos2 {
        let base_scale = 50.0 * self.zoom;
        let (sin_o, cos_o) = self.camera_orbit.sin_cos();
        let rot_x = pos[0] * cos_o - pos[2] * sin_o;

        egui::pos2(
            self.center.x + (rot_x * base_scale),
            self.ground_y - (pos[1] * base_scale),
        )
    }

    /// Dessine un maillage en fil de fer à la position/orientation données.
    pub fn draw_mesh(&self, mesh: &Mesh3D, position: Vec3, rotation: Rotation3D, color: egui::Color32) {
        self.draw_mesh_with_stroke(mesh, position, rotation, egui::Stroke::new(1.0, color));
    }

    /// Variante de `draw_mesh` avec un `Stroke` complet (épaisseur incluse).
    pub fn draw_mesh_with_stroke(
        &self,
        mesh: &Mesh3D,
        position: Vec3,
        rotation: Rotation3D,
        stroke: egui::Stroke,
    ) {
        for tri in &mesh.triangles {
            let screen: Vec<egui::Pos2> = tri
                .iter()
                .map(|&idx| {
                    let r = rotation.rotate_point(mesh.vertices[idx]);
                    let world = [r[0] + position[0], r[1] + position[1], r[2] + position[2]];
                    self.world_to_screen(world)
                })
                .collect();

            self.painter.line_segment([screen[0], screen[1]], stroke);
            self.painter.line_segment([screen[1], screen[2]], stroke);
            self.painter.line_segment([screen[2], screen[0]], stroke);
        }
    }

    /// Dessine le repère d'axes XYZ (Rouge=X, Vert=Y, Bleu=Z) à une position.
    pub fn draw_axes(&self, position: Vec3, rotation: Rotation3D, length: f32) {
        let origin = position;
        let tip = |axis: Vec3| {
            let r = rotation.rotate_point(axis);
            self.world_to_screen([origin[0] + r[0], origin[1] + r[1], origin[2] + r[2]])
        };

        let p_origin = self.world_to_screen(origin);
        let p_x = tip([length, 0.0, 0.0]);
        let p_y = tip([0.0, length, 0.0]);
        let p_z = tip([0.0, 0.0, length]);

        self.painter.line_segment([p_origin, p_x], egui::Stroke::new(2.5, egui::Color32::RED));
        self.painter.line_segment([p_origin, p_y], egui::Stroke::new(2.5, egui::Color32::GREEN));
        self.painter.line_segment([p_origin, p_z], egui::Stroke::new(2.5, egui::Color32::BLUE));
    }

    /// Dessine l'historique d'une trajectoire (polyligne reliant `points`).
    pub fn draw_trail(&self, points: &[Vec3], color: egui::Color32) {
        if points.len() < 2 {
            return;
        }
        let stroke = egui::Stroke::new(1.5, color);
        for w in points.windows(2) {
            let p1 = self.world_to_screen(w[0]);
            let p2 = self.world_to_screen(w[1]);
            self.painter.line_segment([p1, p2], stroke);
        }
    }

    /// Dessine un graphique 2D (courbe de valeurs) dans un rectangle donné
    /// de l'écran, indépendamment de la caméra 3D.
    pub fn draw_chart(&self, rect: egui::Rect, values: &[f32], min_val: f32, max_val: f32, color: egui::Color32) {
        if values.len() < 2 || (max_val - min_val).abs() < f32::EPSILON {
            return;
        }

        self.painter.rect_filled(rect, 4.0, egui::Color32::from_black_alpha(150));
        self.painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

        let (width, height, len) = (rect.width(), rect.height(), values.len());
        let stroke = egui::Stroke::new(1.5, color);

        let point_at = |i: usize| {
            let x_norm = i as f32 / (len - 1) as f32;
            let y_norm = ((values[i] - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
            egui::pos2(rect.min.x + x_norm * width, rect.max.y - y_norm * height)
        };

        for i in 0..len - 1 {
            self.painter.line_segment([point_at(i), point_at(i + 1)], stroke);
        }
    }
}