//! Moteur de rendu haut niveau : alloue la zone de dessin, gère la caméra,
//! dessine le sol/la grille, puis délègue le dessin de la scène à l'appelant.

use crate::camera::OrbitCamera;
use crate::render::Render3D;
use eframe::egui;

/// Options d'apparence de la grille au sol.
///
/// Marquée `#[non_exhaustive]` : de nouveaux champs pourront être ajoutés
/// sans casser la compatibilité. Hors de la crate, pars de `default()` puis
/// utilise les méthodes `with_*` ou modifie les champs :
///
/// ```ignore
/// let grid = GridStyle::default().with_step_px(30.0).with_half_lines(20);
/// ```
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct GridStyle {
    /// Couleur des lignes de la grille.
    pub line_color: egui::Color32,
    /// Couleur de la ligne de sol.
    pub ground_color: egui::Color32,
    /// Couleur de fond de la zone de rendu.
    pub background_color: egui::Color32,
    /// Espacement des lignes (pixels, avant zoom).
    pub step_px: f32,
    /// Nombre de lignes de part et d'autre du centre.
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
    /// Change la couleur des lignes de la grille.
    #[must_use]
    pub fn with_line_color(mut self, color: egui::Color32) -> Self {
        self.line_color = color;
        self
    }

    /// Change la couleur de la ligne de sol.
    #[must_use]
    pub fn with_ground_color(mut self, color: egui::Color32) -> Self {
        self.ground_color = color;
        self
    }

    /// Change la couleur de fond.
    #[must_use]
    pub fn with_background_color(mut self, color: egui::Color32) -> Self {
        self.background_color = color;
        self
    }

    /// Change l'espacement des lignes (pixels, avant zoom).
    #[must_use]
    pub fn with_step_px(mut self, step_px: f32) -> Self {
        self.step_px = step_px;
        self
    }

    /// Change le nombre de lignes de part et d'autre du centre.
    #[must_use]
    pub fn with_half_lines(mut self, half_lines: i32) -> Self {
        self.half_lines = half_lines;
        self
    }
}

/// Moteur de rendu 3D : combine la caméra orbitale et l'affichage de la scène.
#[derive(Default)]
pub struct Engine3D {
    /// Caméra orbitale (angle, zoom, panoramique).
    pub camera: OrbitCamera,
    /// Apparence de la grille au sol.
    pub grid: GridStyle,
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