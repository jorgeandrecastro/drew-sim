//! Exemple : visualiseur orbital affichant un cube animé, son repère XYZ,
//! sa trajectoire et un petit graphe de télémétrie (altitude simulée).
//!
//! Lancer avec :
//!
//! ```text
//! cargo run --example orbit_viewer --release
//! cargo run --example orbit_viewer --release -- chemin/vers/modele.stl   # feature `stl`
//! ```
//!
//! Sert aussi de test manuel pour les nouveautés de la 0.4 : attitude par
//! interpolation de quaternions (`Rotation3D::slerp`), affichage du quaternion
//! (`Quat`), arêtes uniques (`Mesh3D::edges`), `OrbitCamera::reset`,
//! `GridStyle::with_*`.
//!
//! Écrit pour eframe/egui 0.35 : `App::ui(&mut self, ui: &mut egui::Ui, ...)`
//! remplace l'ancien `App::update(&mut self, ctx: &egui::Context, ...)`, et
//! les panels s'affichent maintenant via `.show(ui, ...)` plutôt que
//! `.show(ctx, ...)`. `SidePanel` a été remplacé par `Panel` (unifié avec
//! `TopBottomPanel`).

use drew_sim::egui;
use drew_sim::{Engine3D, GridStyle, Mesh3D, Quat, Rotation3D, TelemetryState, TrailBuffer};

const HISTORY_LEN: usize = 200;

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
    /// `false` : angles d'Euler animés directement ; `true` : interpolation
    /// sphérique (slerp) entre deux attitudes clés.
    slerp_attitude: bool,
}

impl OrbitViewerApp {
    fn new(mesh: Mesh3D) -> Self {
        let mut engine = Engine3D::default();
        engine.grid = GridStyle::default().with_half_lines(12);

        Self {
            engine,
            mesh,
            state: TelemetryState::default(),
            trail: TrailBuffer::new(HISTORY_LEN),
            altitude_history: Vec::with_capacity(HISTORY_LEN),
            time: 0.0,
            orbit_speed: 1.0,
            orbit_radius: 2.0,
            paused: false,
            slerp_attitude: false,
        }
    }

    /// Attitude à l'instant courant selon le mode choisi.
    fn attitude(&self) -> Rotation3D {
        let t = self.time;
        if self.slerp_attitude {
            let a = Rotation3D::IDENTITY;
            let b = Rotation3D::new(0.6, 0.9, 2.5);
            a.slerp(b, 0.5 + 0.5 * t.sin())
        } else {
            Rotation3D::new(
                (t * 2.0).sin() * 0.2, // Léger roulis
                (t * 1.5).cos() * 0.3, // Tangage
                t * 0.5,               // Lacet continu
            )
        }
    }
}

#[cfg(feature = "stl")]
fn load_mesh() -> Mesh3D {
    if let Some(path) = std::env::args().nth(1) {
        match Mesh3D::from_stl_file(&path) {
            Ok(mesh) => {
                eprintln!("STL chargé : {path} ({} triangles)", mesh.triangle_count());
                return mesh;
            }
            Err(e) => eprintln!("Impossible de charger {path} : {e} (cube par défaut)"),
        }
    }
    Mesh3D::unit_cube(1.2)
}

#[cfg(not(feature = "stl"))]
fn load_mesh() -> Mesh3D {
    Mesh3D::unit_cube(1.2)
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

            self.state.rotation = self.attitude();

            // Mise à jour des historiques
            self.trail.push(self.state.position);

            self.altitude_history.push(self.state.position[1]);
            if self.altitude_history.len() > HISTORY_LEN {
                let excess = self.altitude_history.len() - HISTORY_LEN;
                self.altitude_history.drain(..excess);
            }
        }

        // 2. Panneau de contrôle latéral (IHM)
        egui::Panel::left("control_panel")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.heading("🛰️ Orbit Viewer");
                ui.small("Station de sol & Télémétrie");
                ui.separator();

                // Contrôles de simulation
                ui.label("Contrôles de l'orbite :");
                ui.checkbox(&mut self.paused, "Pause simulation");
                ui.add(egui::Slider::new(&mut self.orbit_speed, 0.1..=3.0).text("Vitesse"));
                ui.add(egui::Slider::new(&mut self.orbit_radius, 0.5..=4.0).text("Rayon"));
                ui.checkbox(&mut self.slerp_attitude, "Attitude par slerp (quaternions)");

                ui.horizontal(|ui| {
                    if ui.button("🗑️ Effacer la trajectoire").clicked() {
                        self.trail.clear();
                        self.altitude_history.clear();
                    }
                    if ui.button("🎥 Reset caméra").clicked() {
                        self.engine.camera.reset();
                    }
                });

                ui.separator();

                // Telemetry Data Box
                ui.label("Données en direct :");
                let q = Quat::from_euler(self.state.rotation);
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.monospace(format!("Pos X: {:+.2} m", self.state.position[0]));
                    ui.monospace(format!("Alt Y: {:+.2} m", self.state.position[1]));
                    ui.monospace(format!("Pos Z: {:+.2} m", self.state.position[2]));
                    ui.separator();
                    ui.monospace(format!("Roll:  {:+.2} rad", self.state.rotation.roll));
                    ui.monospace(format!("Pitch: {:+.2} rad", self.state.rotation.pitch));
                    ui.monospace(format!("Yaw:   {:+.2} rad", self.state.rotation.yaw));
                    ui.separator();
                    ui.monospace(format!("q.w: {:+.3}  q.x: {:+.3}", q.w, q.x));
                    ui.monospace(format!("q.y: {:+.3}  q.z: {:+.3}", q.y, q.z));
                });

                ui.separator();
                ui.label("Maillage :");
                ui.small(format!(
                    "{} sommets, {} triangles, {} arêtes uniques",
                    self.mesh.vertices.len(),
                    self.mesh.triangle_count(),
                    self.mesh.edges().len()
                ));

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
                // Rendu du maillage fil de fer
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

    let mesh = load_mesh();

    eframe::run_native(
        "drew-sim — orbit viewer",
        options,
        Box::new(move |_cc| Ok(Box::new(OrbitViewerApp::new(mesh)))),
    )
}