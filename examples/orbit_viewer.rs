//! Exemple : visualiseur orbital affichant un cube animé, son repère XYZ,
//! sa trajectoire et un petit graphe de télémétrie (altitude simulée).
//!
//! Lancer avec : `cargo run --example orbit_viewer --release`
//!
//! Écrit pour eframe/egui 0.35 : `App::ui(&mut self, ui: &mut egui::Ui, ...)`
//! remplace l'ancien `App::update(&mut self, ctx: &egui::Context, ...)`, et
//! les panels s'affichent maintenant via `.show(ui, ...)` plutôt que
//! `.show(ctx, ...)`. `SidePanel` a été remplacé par `Panel` (unifié avec
//! `TopBottomPanel`).

use drew_sim::egui;
use drew_sim::{Engine3D, Mesh3D, Rotation3D, TelemetryState, TrailBuffer};

struct OrbitViewerApp {
    engine: Engine3D,
    mesh: Mesh3D,
    state: TelemetryState,
    trail: TrailBuffer,
    altitude_history: Vec<f32>,
    time: f32,

    // Paramètres configurables en direct dans le panneau
    orbit_speed: f32,
    orbit_radius: f32,
    paused: bool,
}

impl Default for OrbitViewerApp {
    fn default() -> Self {
        Self {
            engine: Engine3D::default(),
            mesh: Mesh3D::unit_cube(1.2),
            state: TelemetryState::default(),
            trail: TrailBuffer::new(200),
            altitude_history: Vec::with_capacity(200),
            time: 0.0,
            orbit_speed: 1.0,
            orbit_radius: 2.0,
            paused: false,
        }
    }
}

impl eframe::App for OrbitViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 1. Simulation physique / télémétrie
        if !self.paused {
            let dt = ctx.input(|i| i.stable_dt).min(0.1); // Pas de temps fluide
            self.time += dt * self.orbit_speed;

            // Orbite circulaire avec oscillation verticale (altitude)
            self.state.position = [
                self.time.cos() * self.orbit_radius,
                (self.time * 1.5).sin() * 0.6,
                self.time.sin() * self.orbit_radius,
            ];

            // Attachement d'attitude (Roll, Pitch, Yaw dynamiques)
            self.state.rotation = Rotation3D::new(
                (self.time * 2.0).sin() * 0.2, // Léger roulis
                (self.time * 1.5).cos() * 0.3, // Tangage
                self.time * 0.5,               // Lacet continu
            );

            // Mise à jour des historiques
            self.trail.push(self.state.position);

            self.altitude_history.push(self.state.position[1]);
            if self.altitude_history.len() > 200 {
                self.altitude_history.remove(0);
            }
        }

        // 2. Panneau de contrôle latéral (IHM)
        egui::Panel::left("control_panel")
            .resizable(true)
            .default_size(240.0)
            .show(ui, |ui| {
                ui.heading("🛰️ Orbit Viewer");
                ui.small("Station de sol & Télémétrie");
                ui.separator();

                // Contrôles de simulation
                ui.label("Contrôles de l'orbite :");
                ui.checkbox(&mut self.paused, "Pause simulation");
                ui.add(egui::Slider::new(&mut self.orbit_speed, 0.1..=3.0).text("Vitesse"));
                ui.add(egui::Slider::new(&mut self.orbit_radius, 0.5..=4.0).text("Rayon"));

                if ui.button("🗑️ Effacer la trajectoire").clicked() {
                    self.trail.clear();
                    self.altitude_history.clear();
                }

                ui.separator();

                // Telemetry Data Box
                ui.label("Données en direct :");
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.monospace(format!("Pos X: {:+.2} m", self.state.position[0]));
                    ui.monospace(format!("Alt Y: {:+.2} m", self.state.position[1]));
                    ui.monospace(format!("Pos Z: {:+.2} m", self.state.position[2]));
                    ui.separator();
                    ui.monospace(format!("Pitch: {:+.2} rad", self.state.rotation.pitch));
                    ui.monospace(format!("Yaw:   {:+.2} rad", self.state.rotation.yaw));
                });

                ui.separator();
                ui.collapsing("🖱️ Contrôles Caméra", |ui| {
                    ui.small("• Clic Gauche + Glisser : Orbite");
                    ui.small("• Molette : Zoom");
                    ui.small("• Clic Droit / Molette : Panoramique");
                });
            });

        // 3. Zone de rendu 3D
        egui::CentralPanel::default().show(ui, |ui| {
            // Calcul préalable du rectangle du graphe pour éviter tout conflit de borrow avec `ui`
            let avail_rect = ui.available_rect_before_wrap();
            let chart_rect = egui::Rect::from_min_size(
                egui::pos2(avail_rect.max.x - 230.0, avail_rect.min.y + 15.0),
                egui::vec2(215.0, 90.0),
            );

            self.engine.render(ui, |r| {
                // Rendu du maillage fil de fer du cube
                r.draw_mesh(
                    &self.mesh,
                    self.state.position,
                    self.state.rotation,
                    egui::Color32::from_rgb(100, 200, 255),
                );

                // Repère d'axes XYZ sur l'objet
                r.draw_axes(self.state.position, self.state.rotation, 1.2);

                // Trajectoire historique
                r.draw_trail(self.trail.as_slice(), egui::Color32::GOLD);

                // Overlay Graphe 2D de télémétrie (Altitude)
                r.draw_chart(
                    chart_rect,
                    &self.altitude_history,
                    -1.0, // Min Y
                    1.0,  // Max Y
                    egui::Color32::GREEN,
                );
            });
        });

        // Demande le rafraîchissement continu de l'affichage
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 680.0])
            .with_title("drew-sim Orbit Viewer Example"),
        ..Default::default()
    };

    eframe::run_native(
        "drew-sim — orbit viewer",
        options,
        Box::new(|_cc| Ok(Box::new(OrbitViewerApp::default()))),
    )
}