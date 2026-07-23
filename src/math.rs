//! Primitives mathématiques : vecteurs 3D et rotations Euler.

/// Vecteur / point 3D (x, y, z).
pub type Vec3 = [f32; 3];

/// Addition de deux vecteurs.
#[inline]
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Soustraction de deux vecteurs.
#[inline]
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Mise à l'échelle d'un vecteur par un scalaire.
#[inline]
pub fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Représente les 3 angles de rotation en radians (Roulis, Tangage, Lacet).
///
/// Convention : Roll autour de Z, Pitch autour de X, Yaw autour de Y,
/// appliqués dans cet ordre (Roll puis Pitch puis Yaw).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rotation3D {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Rotation3D {
    pub const IDENTITY: Self = Self { roll: 0.0, pitch: 0.0, yaw: 0.0 };

    pub fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self { roll, pitch, yaw }
    }

    /// Applique la rotation Roll -> Pitch -> Yaw à un point 3D.
    pub fn rotate_point(&self, p: Vec3) -> Vec3 {
        // Roll (Z)
        let (sin_r, cos_r) = self.roll.sin_cos();
        let x1 = p[0] * cos_r - p[1] * sin_r;
        let y1 = p[0] * sin_r + p[1] * cos_r;
        let z1 = p[2];

        // Pitch (X)
        let (sin_p, cos_p) = self.pitch.sin_cos();
        let x2 = x1;
        let y2 = y1 * cos_p - z1 * sin_p;
        let z2 = y1 * sin_p + z1 * cos_p;

        // Yaw (Y)
        let (sin_y, cos_y) = self.yaw.sin_cos();
        let x3 = x2 * cos_y + z2 * sin_y;
        let y3 = y2;
        let z3 = -x2 * sin_y + z2 * cos_y;

        [x3, y3, z3]
    }

    /// Compose deux rotations (approximation par angles d'Euler cumulés).
    /// Pour une composition rigoureuse, préférer une future implémentation
    /// via quaternions (voir `embedded-quaternion-f32`).
    pub fn compose(&self, other: Rotation3D) -> Rotation3D {
        Rotation3D {
            roll: self.roll + other.roll,
            pitch: self.pitch + other.pitch,
            yaw: self.yaw + other.yaw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_rotation_is_noop() {
        let p = [1.0, 2.0, 3.0];
        let r = Rotation3D::IDENTITY;
        let out = r.rotate_point(p);
        for i in 0..3 {
            assert!((out[i] - p[i]).abs() < 1e-6);
        }
    }
}