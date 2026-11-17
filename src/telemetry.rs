//! Source-agnostic telemetry contract (sensors, simulation, or log playback).

use crate::math::{Rotation3D, Vec3};

/// Position and orientation of an object at a given instant.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TelemetryState {
    /// Position in the scene.
    pub position: Vec3,
    /// Orientation (Euler angles; see the conventions in [`crate::math`]).
    pub rotation: Rotation3D,
}

impl TelemetryState {
    /// Creates a telemetry state.
    #[must_use]
    pub const fn new(position: Vec3, rotation: Rotation3D) -> Self {
        Self { position, rotation }
    }
}

/// Bounded history of positions, useful for drawing a trail (`draw_trail`)
/// without allowing the buffer to grow indefinitely.
///
/// Keeps the last `capacity` positions. Internally, storage may reach
/// `2 * capacity` before being compacted, making `push` amortized O(1)
/// (instead of shifting the entire buffer on every insertion).
#[derive(Clone, Debug)]
pub struct TrailBuffer {
    points: Vec<Vec3>,
    capacity: usize,
}

impl TrailBuffer {
    /// Creates a history holding at most `capacity` positions
    /// (when `capacity == 0`, no positions are retained).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self { points: Vec::with_capacity(capacity), capacity }
    }

    /// Adds a position; the oldest is discarded if the buffer is full.
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

    /// Retained positions, from oldest to newest.
    #[must_use]
    pub fn as_slice(&self) -> &[Vec3] {
        &self.points[self.points.len().saturating_sub(self.capacity)..]
    }

    /// Number of retained positions (at most `capacity`).
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// `true` if no positions are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Maximum capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clears the history.
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