#![warn(missing_docs)]
//! # drew-sim
//!
//! Librairie de rendu 3D immédiat pour applications [`eframe`]/[`egui`].
//!
//! Pensée pour la visualisation de télémétrie (drones, robots, capteurs
//! d'orientation) : chargement de maillages `.stl`, caméra orbitale avec
//! zoom/pan à la souris, et primitives de dessin (maillage fil de fer,
//! repère XYZ, trajectoire, graphe de valeurs).
//!
//! ## Exemple minimal
//!
//! ```no_run
//! use drew_sim::{Engine3D, Mesh3D, TelemetryState};
//! use drew_sim::egui;
//!
//! struct App {
//!     engine: Engine3D,
//!     mesh: Mesh3D,
//!     state: TelemetryState,
//! }
//!
//! impl eframe::App for App {
//!     fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
//!         egui::CentralPanel::default().show(ui, |ui| {
//!             self.engine.render(ui, |r| {
//!                 r.draw_mesh(&self.mesh, self.state.position, self.state.rotation, egui::Color32::WHITE);
//!                 r.draw_axes(self.state.position, self.state.rotation, 1.5);
//!             });
//!         });
//!     }
//! }
//! ```

pub mod camera;
pub mod engine;
pub mod math;
pub mod mesh;
pub mod quat;
pub mod render;
pub mod telemetry;

pub use camera::OrbitCamera;
pub use engine::{Engine3D, GridStyle};
pub use math::{Rotation3D, Vec3};
pub use mesh::{Mesh3D, MeshError};
pub use quat::Quat;
pub use render::Render3D;
pub use telemetry::{TelemetryState, TrailBuffer};

// Ré-export pratique pour les consommateurs de la librairie.
pub use eframe;
pub use eframe::egui;