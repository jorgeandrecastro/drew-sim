//! Contrat de télémétrie agnostique de la source (capteurs, simulation, replay).

use crate::math::{Rotation3D, Vec3};

/// Position et orientation d'un objet à un instant donné.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TelemetryState {
    /// Position dans la scène.
    pub position: Vec3,
    /// Orientation (angles d'Euler, voir la convention dans [`crate::math`]).
    pub rotation: Rotation3D,
}

impl TelemetryState {
    /// Construit un état de télémétrie.
    #[must_use]
    pub const fn new(position: Vec3, rotation: Rotation3D) -> Self {
        Self { position, rotation }
    }
}

/// Historique borné de positions, utile pour tracer une trajectoire
/// (`draw_trail`) sans laisser le buffer grandir indéfiniment.
///
/// Garde les `capacity` dernières positions. En interne, le stockage peut
/// atteindre `2 * capacity` avant d'être compacté, ce qui rend `push`
/// amorti O(1) (au lieu de décaler tout le buffer à chaque insertion).
#[derive(Clone, Debug)]
pub struct TrailBuffer {
    points: Vec<Vec3>,
    capacity: usize,
}

impl TrailBuffer {
    /// Crée un historique de `capacity` positions maximum
    /// (avec `capacity == 0`, rien n'est jamais conservé).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self { points: Vec::with_capacity(capacity), capacity }
    }

    /// Ajoute une position ; la plus ancienne est oubliée si le buffer est plein.
    pub fn push(&mut self, p: Vec3) {
        if self.capacity == 0 {
            return;
        }
        self.points.push(p);
        if self.points.len() >= self.capacity.saturating_mul(2) {
            let excess = self.points.len() - self.capacity;
            self.points.drain(..excess);
        }
    }

    /// Les positions conservées, de la plus ancienne à la plus récente.
    #[must_use]
    pub fn as_slice(&self) -> &[Vec3] {
        &self.points[self.points.len().saturating_sub(self.capacity)..]
    }

    /// Nombre de positions conservées (au plus `capacity`).
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// `true` si aucune position n'est conservée.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Capacité maximale.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Vide l'historique.
    pub fn clear(&mut self) {
        self.points.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(i: usize) -> Vec3 {
        [i as f32, 0.0, 0.0]
    }

    #[test]
    fn keeps_last_capacity_points_in_order() {
        let mut t = TrailBuffer::new(3);
        for i in 0..10 {
            t.push(p(i));
            assert!(t.len() <= 3);
        }
        assert_eq!(t.as_slice(), &[p(7), p(8), p(9)]);
    }

    #[test]
    fn under_capacity_keeps_everything() {
        let mut t = TrailBuffer::new(5);
        t.push(p(0));
        t.push(p(1));
        assert_eq!(t.as_slice(), &[p(0), p(1)]);
        assert!(!t.is_empty());
    }

    #[test]
    fn zero_capacity_does_not_panic() {
        let mut t = TrailBuffer::new(0);
        t.push(p(0));
        assert!(t.as_slice().is_empty());
        assert!(t.is_empty());
    }

    #[test]
    fn clear_empties() {
        let mut t = TrailBuffer::new(2);
        t.push(p(0));
        t.clear();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn internal_storage_stays_bounded() {
        let mut t = TrailBuffer::new(100);
        for i in 0..10_000 {
            t.push(p(i));
        }
        assert!(t.points.len() < 200);
        assert_eq!(t.as_slice().len(), 100);
        assert_eq!(t.as_slice()[99], p(9_999));
    }
}