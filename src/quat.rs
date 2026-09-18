//! Quaternions unitaires : format natif des IMU et autopilotes
//! (PX4, ArduPilot, BNO055…), sans gimbal lock.
//!
//! Même convention que [`Rotation3D`] : repère main droite, et
//! `Quat::from_euler(r)` représente exactement `Ry(yaw) * Rx(pitch) * Rz(roll)`.

use crate::math::{mat3_mul_vec, normalize, Mat3, Rotation3D, Vec3};
use std::ops::Mul;

/// Quaternion `w + xi + yj + zk`. Les fonctions de rotation supposent
/// (ou imposent) une norme de 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    /// Partie réelle.
    pub w: f32,
    /// Composante i.
    pub x: f32,
    /// Composante j.
    pub y: f32,
    /// Composante k.
    pub z: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    /// Rotation nulle.
    pub const IDENTITY: Self = Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 };

    /// Construit un quaternion (non normalisé) à partir de ses composantes.
    #[must_use]
    #[inline]
    pub const fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        Self { w, x, y, z }
    }

    /// Rotation de `angle` radians autour de `axis` (l'axe est normalisé).
    #[must_use]
    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let a = normalize(axis);
        let (s, c) = (angle * 0.5).sin_cos();
        Self { w: c, x: a[0] * s, y: a[1] * s, z: a[2] * s }
    }

    /// Convertit des angles d'Euler (`Ry(yaw) * Rx(pitch) * Rz(roll)`).
    #[must_use]
    pub fn from_euler(r: Rotation3D) -> Self {
        let qz = Self::from_axis_angle([0.0, 0.0, 1.0], r.roll);
        let qx = Self::from_axis_angle([1.0, 0.0, 0.0], r.pitch);
        let qy = Self::from_axis_angle([0.0, 1.0, 0.0], r.yaw);
        qy * qx * qz
    }

    /// Convertit en angles d'Euler normalisés (voir [`Rotation3D::from_matrix`]
    /// pour le comportement au gimbal lock).
    #[must_use]
    pub fn to_euler(self) -> Rotation3D {
        Rotation3D::from_matrix(self.to_matrix())
    }

    /// Produit scalaire 4D.
    #[must_use]
    #[inline]
    pub fn dot(self, o: Self) -> f32 {
        self.w * o.w + self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Norme du quaternion.
    #[must_use]
    #[inline]
    pub fn norm(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Renvoie le quaternion de norme 1 (ou l'identité si la norme est
    /// quasi nulle). À appeler régulièrement si l'on intègre des vitesses
    /// angulaires, pour éviter la dérive.
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

    /// Conjugué (`w, -x, -y, -z`) : l'inverse pour un quaternion unitaire.
    #[must_use]
    #[inline]
    pub fn conjugate(self) -> Self {
        Self { w: self.w, x: -self.x, y: -self.y, z: -self.z }
    }

    /// Inverse (`conjugé / norme²`). Identité si la norme est quasi nulle.
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

    /// Compose deux rotations : `self` d'abord, puis `other`
    /// (même sémantique que [`Rotation3D::compose`]).
    #[must_use]
    #[inline]
    pub fn compose(self, other: Self) -> Self {
        other * self
    }

    /// Matrice de rotation 3x3 (le quaternion est normalisé au préalable).
    #[must_use]
    pub fn to_matrix(self) -> Mat3 {
        let Self { w, x, y, z } = self.normalize();
        [
            [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
            [2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x)],
            [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)],
        ]
    }

    /// Applique la rotation à un point.
    #[must_use]
    pub fn rotate_point(self, p: Vec3) -> Vec3 {
        mat3_mul_vec(&self.to_matrix(), p)
    }

    /// Interpolation sphérique (t = 0 → `self`, t = 1 → `other`),
    /// par le chemin le plus court. Repli sur une interpolation linéaire
    /// normalisée quand les deux quaternions sont quasi identiques.
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

/// Produit de Hamilton : `a * b` applique `b` d'abord, puis `a`
/// (comme le produit de matrices).
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
        let b = Quat::new(-1.0, 0.0, 0.0, 0.0); // même rotation, signe opposé
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