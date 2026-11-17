//! Example: an orbital viewer showing an animated cube, its XYZ axes,
//! its trail, and a small telemetry chart (simulated altitude).
//!
//! Run with:
//!
//! ```text
//! cargo run --example orbit_viewer --release
//! cargo run --example orbit_viewer --release -- path/to/model.stl   # `stl` feature
//! ```
//!
//! It also serves as a manual check for 0.4 features: attitude interpolation
//! using quaternions (`Rotation3D::slerp`), quaternion display (`Quat`), unique
//! edges (`Mesh3D::edges`), `OrbitCamera::reset`, and `GridStyle::with_*`.
//!
//! Written for eframe/egui 0.35: `App::ui(&mut self, ui: &mut egui::Ui, ...)`
//! replaces the former `App::update(&mut self, ctx: &egui::Context, ...)`, and
//! panels are now displayed with `.show(ui, ...)` rather than `.show(ctx, ...)`.
//! `SidePanel` has been replaced by `Panel` (unified with `TopBottomPanel`).

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

    // Parameters that can be adjusted live in the panel.
    orbit_speed: f32,
    orbit_radius: f32,
    paused: bool,
    /// `false`: animate Euler angles directly; `true`: use spherical
    /// interpolation (slerp) between two key attitudes.
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

    /// Attitude at the current time, according to the selected mode.
    fn attitude(&self) -> Rotation3D {
        let t = self.time;
        if self.slerp_attitude {
            let a = Rotation3D::IDENTITY;
            let b = Rotation3D::new(0.6, 0.9, 2.5);
            a.slerp(b, 0.5 + 0.5 * t.sin())
        } else {
            Rotation3D::new(
                (t * 2.0).sin() * 0.2, // Slight roll
                (t * 1.5).cos() * 0.3, // Pitch
                t * 0.5,               // Continuous yaw
            )
        }
    }
}

#[cfg(feature = "stl")]
fn load_mesh() -> Mesh3D {
    if let Some(path) = std::env::args().nth(1) {
        match Mesh3D::from_stl_file(&path) {
            Ok(mesh) => {
                eprintln!("Loaded STL: {path} ({} triangles)", mesh.triangle_count());
                return mesh;
            }
            Err(e) => eprintln!("Failed to load {path}: {e} (using default cube)"),
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

        // 1. Physics simulation / telemetry
        if !self.paused {
            let dt = ctx.input(|i| i.stable_dt).min(0.1); // Smooth time step
            self.time += dt * self.orbit_speed;

            // Circular orbit with vertical oscillation (altitude)
            self.state.position = [
                self.time.cos() * self.orbit_radius,
                (self.time * 1.5).sin() * 0.6,
                self.time.sin() * self.orbit_radius,
            ];

            self.state.rotation = self.attitude();

            // Update histories
            self.trail.push(self.state.position);

            self.altitude_history.push(self.state.position[1]);
            if self.altitude_history.len() > HISTORY_LEN {
                let excess = self.altitude_history.len() - HISTORY_LEN;
                self.altitude_history.drain(..excess);
            }
        }

        // 2. Side control panel (UI)
        egui::Panel::left("control_panel")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.heading("🛰️ Orbit Viewer");
                ui.small("Ground Station & Telemetry");
                ui.separator();

                // Simulation controls
                ui.label("Orbit controls:");
                ui.checkbox(&mut self.paused, "Pause simulation");
                ui.add(egui::Slider::new(&mut self.orbit_speed, 0.1..=3.0).text("Speed"));
                ui.add(egui::Slider::new(&mut self.orbit_radius, 0.5..=4.0).text("Radius"));
                ui.checkbox(&mut self.slerp_attitude, "Slerp attitude (quaternions)");

                ui.horizontal(|ui| {
                    if ui.button("🗑️ Clear trail").clicked() {
                        self.trail.clear();
                        self.altitude_history.clear();
                    }
                    if ui.button("🎥 Reset camera").clicked() {
                        self.engine.camera.reset();
                    }
                });

                ui.separator();

                // Telemetry data
                ui.label("Live data:");
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
                ui.label("Mesh:");
                ui.small(format!(
                    "{} vertices, {} triangles, {} unique edges",
                    self.mesh.vertices.len(),
                    self.mesh.triangle_count(),
                    self.mesh.edges().len()
                ));

                ui.separator();
                ui.collapsing("🖱️ Camera controls", |ui| {
                    ui.small("• Left-click and drag: orbit");
                    ui.small("• Scroll wheel: zoom");
                    ui.small("• Right/middle-click and drag: pan");
                });
            });

        // 3. 3D rendering area
        egui::CentralPanel::default().show(ui, |ui| {
            // Calculate the chart rectangle first to avoid a borrow conflict with `ui`.
            let avail_rect = ui.available_rect_before_wrap();
            let chart_rect = egui::Rect::from_min_size(
                egui::pos2(avail_rect.max.x - 230.0, avail_rect.min.y + 15.0),
                egui::vec2(215.0, 90.0),
            );

            self.engine.render(ui, |r| {
                // Render the wireframe mesh
                r.draw_mesh(
                    &self.mesh,
                    self.state.position,
                    self.state.rotation,
                    egui::Color32::from_rgb(100, 200, 255),
                );

                // XYZ axes on the object
                r.draw_axes(self.state.position, self.state.rotation, 1.2);

                // Trail history
                r.draw_trail(self.trail.as_slice(), egui::Color32::GOLD);

                // 2D telemetry chart overlay (altitude)
                r.draw_chart(
                    chart_rect,
                    &self.altitude_history,
                    -1.0, // Min Y
                    1.0,  // Max Y
                    egui::Color32::GREEN,
                );
            });
        });

        // Request continuous repainting.
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