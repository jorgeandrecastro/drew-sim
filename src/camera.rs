//! Caméra orbitale et gestion de l'entrée utilisateur (zoom, pan, orbite).

use eframe::egui;

/// Bornes de zoom par défaut.
pub const DEFAULT_ZOOM_RANGE: (f32, f32) = (0.2, 5.0);

/// État de la caméra orbitale : angle d'orbite, zoom, décalage de panoramique.
#[derive(Clone, Copy, Debug)]
pub struct OrbitCamera {
    /// Angle d'orbite horizontal (radians) autour de l'axe vertical.
    pub orbit: f32,
    /// Facteur de zoom, borné par `zoom_range`.
    pub zoom: f32,
    /// Décalage de panoramique (pixels écran).
    pub pan_offset: egui::Vec2,
    /// Bornes `(min, max)` du zoom.
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
    /// Remet l'orbite, le zoom et le panoramique à leurs valeurs par défaut
    /// (les bornes de zoom personnalisées sont conservées).
    pub fn reset(&mut self) {
        *self = Self { zoom_range: self.zoom_range, ..Self::default() };
    }

    /// Met à jour la caméra à partir de la réponse d'interaction egui pour
    /// la zone de rendu (molette = zoom, clic droit/milieu + glisser = pan,
    /// clic gauche + glisser = orbite).
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