//! Unit quaternions: the native format for IMUs and autopilots
//! (PX4, ArduPilot, BNO055, etc.), with no gimbal lock.
//!
//! Uses the same conventions as [`Rotation3D`]: a right-handed coordinate
//! system, where `Quat::from_euler(r)` represents exactly
//! `Ry(yaw) * Rx(pitch) * Rz(roll)`.

use crate::math::{mat3_mul_vec, normalize, Mat3, Rotation3D, Vec3};
use std::ops::Mul;

/// Quaternion `w + xi + yj + zk`. Rotation functions assume (or enforce) a
/// unit norm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    /// Real part.
    pub w: f32,
    /// i component.
    pub x: f32,
    /// j component.
    pub y: f32,
    /// k component.
    pub z: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    /// Zero rotation.
    pub const IDENTITY: Self = Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 };

    /// Creates a quaternion (not normalized) from its components.
    #[must_use]
    #[inline]
    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// Rotation of `angle` radians around `axis` (the axis is normalized).
    #[must_use]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let a = normalize(axis);
        let (s, c) = (angle * 0.5).sin_cos();
        Self { w: c, x: a[0] * s, y: a[1] * s, z: a[2] * s }
    }

    /// Converts Euler angles (`Ry(yaw) * Rx(pitch) * Rz(roll)`).
    #[must_use]
    pub fn from_euler(r: Rotation3D) -> Self {
        let qz = Self::from_axis_angle([0.0, 0.0, 1.0], r.roll);
        let qx = Self::from_axis_angle([1.0, 0.0, 0.0], r.pitch);
        let qy = Self::from_axis_angle([0.0, 1.0, 0.0], r.yaw);
        qy * qx * qz
    }

    /// Converts to normalized Euler angles (see [`Rotation3D::from_matrix`]
    /// for behavior at gimbal lock).
    #[must_use]
    pub fn to_euler(self) -> Rotation3D {
        Rotation3D::from_matrix(self.to_matrix())
    }

    /// 4D dot product.
    #[must_use]
    #[inline]
    pub fn dot(self, o: Self) -> f32 {
        self.w * o.w + self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Quaternion norm.
    #[must_use]
    #[inline]
    pub fn norm(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Returns the unit quaternion (or the identity if the norm is near zero).
    /// Call regularly when integrating angular velocities to avoid drift.
    #[must_use]
    pub fn normalize(self) -> Self {
        let n = self.norm();
        if n > f32::EPSILON {
            let k = 1.0 / n;
            Self { w: self.w * k, x: self.x * k, y: self.y * k, z: self.z * k }
        } else {
            Self::IDENTITY
        }
    }

    /// Conjugate (`w, -x, -y, -z`): the inverse of a unit quaternion.
    #[must_use]
    #[inline]
    pub fn conjugate(self) -> Self {
        Self { w: self.w, x: -self.x, y: -self.y, z: -self.z }
    }

    /// Inverse (`conjugate / norm²`). Returns the identity if the norm is near zero.
    #[must_use]
    pub fn inverse(self) -> Self {
        let n2 = self.dot(self);
        if n2 > f32::EPSILON {
            let k = 1.0 / n2;
            let c = self.conjugate();
            Self { w: c.w * k, x: c.x * k, y: c.y * k, z: c.z * k }
        } else {
            Self::IDENTITY
        }
    }

    /// Composes two rotations: `self` first, then `other` (same semantics as
    /// [`Rotation3D::compose`]).
    #[must_use]
    #[inline]
    pub fn compose(self, other: Self) -> Self {
        other * self
    }

    /// 3x3 rotation matrix (the quaternion is normalized first).
    #[must_use]
    pub fn to_matrix(self) -> Mat3 {
        let Self { w, x, y, z } = self.normalize();
        [
            [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
            [2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x)],
            [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)],
        ]
    }

    /// Applies the rotation to a point.
    #[must_use]
    pub fn rotate_point(self, p: Vec3) -> Vec3 {
        mat3_mul_vec(&self.to_matrix(), p)
    }

    /// Spherical interpolation (t = 0 → `self`, t = 1 → `other`) along the
    /// shortest path. Falls back to normalized linear interpolation when the
    /// two quaternions are nearly identical.
    #[must_use]
    pub fn slerp(self, other: Self, t: f32) -> Self {
        let mut b = other;
        let mut d = self.dot(b);
        if d < 0.0 {
            b = Self { w: -b.w, x: -b.x, y: -b.y, z: -b.z };
            d = -d;
        }
        let (wa, wb) = if d > 0.9995 {
            (1.0 - t, t)
        } else {
            let theta = d.clamp(-1.0, 1.0).acos();
            let s = theta.sin();
            (((1.0 - t) * theta).sin() / s, (t * theta).sin() / s)
        };
        Self {
            w: self.w * wa + b.w * wb,
            x: self.x * wa + b.x * wb,
            y: self.y * wa + b.y * wb,
            z: self.z * wa + b.z * wb,
        }
        .normalize()
    }
}

/// Hamilton product: `a * b` applies `b` first, then `a` (like matrix
/// multiplication).
impl Mul for Quat {
    type Output = Quat;
    fn mul(self, o: Quat) -> Quat {
        Quat {
            w: self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
            x: self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
            y: self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
            z: self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
        }
    }
}

impl From<Rotation3D> for Quat {
    fn from(r: Rotation3D) -> Self {
        Self::from_euler(r)
    }
}

impl From<Quat> for Rotation3D {
    fn from(q: Quat) -> Self {
        q.to_euler()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const PROBES: [Vec3; 4] = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, -2.0, 0.5],
    ];

    fn close(a: Vec3, b: Vec3, eps: f32) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < eps, "a={a:?} b={b:?}");
        }
    }

    #[test]
    fn from_euler_matches_rotation3d() {
        for r in [
            Rotation3D::new(0.3, 1.1, -0.7),
            Rotation3D::new(-2.0, -0.4, 3.0),
            Rotation3D::new(4.0, 2.1, -5.0),
        ] {
            let q = Quat::from_euler(r);
            for p in PROBES {
                close(q.rotate_point(p), r.rotate_point(p), 1e-5);
            }
        }
    }

    #[test]
    fn euler_roundtrip_preserves_effect() {
        let r = Rotation3D::new(0.5, -0.8, 2.0);
        let back = Quat::from_euler(r).to_euler();
        for p in PROBES {
            close(back.rotate_point(p), r.rotate_point(p), 1e-4);
        }
    }

    #[test]
    fn compose_matches_rotation3d_compose() {
        let a = Rotation3D::new(0.3, 1.1, -0.7);
        let b = Rotation3D::new(-0.9, 0.4, 1.6);
        let q = Quat::from_euler(a).compose(Quat::from_euler(b));
        for p in PROBES {
            close(q.rotate_point(p), a.compose(b).rotate_point(p), 1e-4);
        }
    }

    #[test]
    fn inverse_cancels_rotation() {
        let q = Quat::from_euler(Rotation3D::new(0.3, 1.1, -0.7));
        let id = q.compose(q.inverse());
        for p in PROBES {
            close(id.rotate_point(p), p, 1e-5);
        }
    }

    #[test]
    fn slerp_endpoints_and_midpoint() {
        let a = Quat::IDENTITY;
        let b = Quat::from_euler(Rotation3D::new(0.0, 0.0, FRAC_PI_2));
        let half = Quat::from_euler(Rotation3D::new(0.0, 0.0, FRAC_PI_2 / 2.0));
        for p in PROBES {
            close(a.slerp(b, 0.0).rotate_point(p), a.rotate_point(p), 1e-5);
            close(a.slerp(b, 1.0).rotate_point(p), b.rotate_point(p), 1e-5);
            close(a.slerp(b, 0.5).rotate_point(p), half.rotate_point(p), 1e-5);
        }
    }

    #[test]
    fn slerp_takes_shortest_path() {
        let a = Quat::IDENTITY;
        let b = Quat::new(-1.0, 0.0, 0.0, 0.0); // Same rotation, opposite sign.
        let mid = a.slerp(b, 0.5);
        for p in PROBES {
            close(mid.rotate_point(p), p, 1e-5);
        }
    }

    #[test]
    fn rotation3d_slerp_via_quaternions() {
        let a = Rotation3D::IDENTITY;
        let b = Rotation3D::new(0.0, 0.0, FRAC_PI_2);
        let mid = a.slerp(b, 0.5);
        for p in PROBES {
            close(
                mid.rotate_point(p),
                Rotation3D::new(0.0, 0.0, FRAC_PI_2 / 2.0).rotate_point(p),
                1e-4,
            );
        }
    }

    #[test]
    fn normalize_handles_zero() {
        assert_eq!(Quat::new(0.0, 0.0, 0.0, 0.0).normalize(), Quat::IDENTITY);
        let n = Quat::new(2.0, 0.0, 0.0, 0.0).normalize();
        assert!((n.norm() - 1.0).abs() < 1e-6);
    }
}