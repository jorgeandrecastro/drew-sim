//! Moteur de rendu haut niveau : alloue la zone de dessin, gère la caméra,
//! dessine le sol/la grille, puis délègue le dessin de la scène à l'appelant.

use crate::camera::OrbitCamera;
use crate::render::Render3D;
use eframe::egui;

/// Options d'apparence de la grille au sol.
#[derive(Clone, Copy, Debug)]
pub struct GridStyle {
    pub line_color: egui::Color32,
    pub ground_color: egui::Color32,
    pub background_color: egui::Color32,
    pub step_px: f32,
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

/// Moteur de rendu 3D : combine la caméra orbitale et l'affichage de la scène.
pub struct Engine3D {
    pub camera: OrbitCamera,
    pub grid: GridStyle,
}

impl Default for Engine3D {
    fn default() -> Self {
        Self { camera: OrbitCamera::default(), grid: GridStyle::default() }
    }
}

impl Engine3D {
    /// Alloue la zone de rendu, met à jour la caméra selon l'entrée utilisateur,
    /// dessine l'arrière-plan/la grille, puis appelle `draw_callback` avec le
    /// contexte de rendu prêt à l'emploi pour dessiner la scène.
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