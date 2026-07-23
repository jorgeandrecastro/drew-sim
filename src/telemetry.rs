//! Contrat de télémétrie agnostique de la source (capteurs, simulation, replay).

use crate::math::{Rotation3D, Vec3};

/// Position et orientation d'un objet à un instant donné.
#[derive(Clone, Copy, Debug, Default)]
pub struct TelemetryState {
    pub position: Vec3,
    pub rotation: Rotation3D,
}

impl TelemetryState {
    pub fn new(position: Vec3, rotation: Rotation3D) -> Self {
        Self { position, rotation }
    }
}

/// Historique borné de positions, utile pour tracer une trajectoire (`draw_trail`)
/// sans laisser le buffer grandir indéfiniment.
#[derive(Clone, Debug)]
pub struct TrailBuffer {
    points: Vec<Vec3>,
    capacity: usize,
}

impl TrailBuffer {
    pub fn new(capacity: usize) -> Self {
        Self { points: Vec::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, p: Vec3) {
        if self.points.len() == self.capacity {
            self.points.remove(0);
        }
        self.points.push(p);
    }

    pub fn as_slice(&self) -> &[Vec3] {
        &self.points
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }
}