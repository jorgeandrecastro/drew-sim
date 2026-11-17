//! Mathematical primitives: 3D vectors and Euler rotations.
//!
//! # Conventions
//!
//! - **Right-handed** coordinate system; angles are in **radians** and positive
//!   in the counterclockwise direction (the right-hand rule around the axis).
//! - `roll` rotates around **Z**, `pitch` around **X**, and `yaw` around **Y**.
//! - Application order: roll, then pitch, then yaw, i.e.
//!   `R = Ry(yaw) * Rx(pitch) * Rz(roll)`.
//! - Y is the scene's vertical axis; Z, the roll axis, points toward the
//!   vehicle's front.
//! - Angles returned by [`Rotation3D::compose`], [`Rotation3D::inverse`], and
//!   [`Rotation3D::from_matrix`] are **normalized**: `pitch` is in
//!   `[-π/2, π/2]`, and `roll` and `yaw` are in `[-π, π]`. The returned triplet
//!   may differ from the input while representing the same rotation.

/// 3D vector / point (x, y, z).
pub type Vec3 = [f32; 3];

/// 3x3 matrix, stored row by row.
pub type Mat3 = [[f32; 3]; 3];

/// 3x3 identity matrix.
pub const IDENTITY_MAT3: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Adds two vectors.
#[must_use]
#[inline]
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Subtracts two vectors.
#[must_use]
#[inline]
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Scales a vector by a scalar.
#[must_use]
#[inline]
pub fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Dot product.
#[must_use]
#[inline]
pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product.
#[must_use]
#[inline]
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Vector magnitude (length).
#[must_use]
#[inline]
pub fn length(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}

/// Normalizes the vector. Returns the original vector if its length is too
/// close to zero (avoids division by zero).
#[must_use]
#[inline]
pub fn normalize(a: Vec3) -> Vec3 {
    let len = length(a);
    if len > f32::EPSILON {
        scale(a, 1.0 / len)
    } else {
        a
    }
}

/// Product of two 3x3 matrices (`a * b`).
#[must_use]
pub fn mat3_mul(a: Mat3, b: Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// Matrix-vector product (`m * v`).
#[must_use]
#[inline]
pub fn mat3_mul_vec(m: &Mat3, v: Vec3) -> Vec3 {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}

/// Transpose of a 3x3 matrix (equal to the inverse for a rotation matrix).
#[must_use]
pub fn mat3_transpose(m: Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Rotation expressed as Euler angles (radians).
///
/// See the module documentation for conventions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rotation3D {
    /// Roll: rotation around Z (radians).
    pub roll: f32,
    /// Pitch: rotation around X (radians).
    pub pitch: f32,
    /// Yaw: rotation around Y (radians).
    pub yaw: f32,
}

impl Rotation3D {
    /// Zero rotation.
    pub const IDENTITY: Self = Self { roll: 0.0, pitch: 0.0, yaw: 0.0 };

    /// Creates a rotation from `roll`, `pitch`, and `yaw` (radians).
    #[must_use]
    #[inline]
    pub const fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self { roll, pitch, yaw }
    }

    /// Applies the rotation (roll, then pitch, then yaw) to a 3D point.
    ///
    /// For many points, prefer [`Self::rotate_points`], which computes the
    /// matrix only once.
    #[must_use]
    #[inline]
    pub fn rotate_point(self, p: Vec3) -> Vec3 {
        mat3_mul_vec(&self.to_matrix(), p)
    }

    /// Applies the rotation to a batch of points (the matrix is computed once).
    #[must_use]
    pub fn rotate_points(self, pts: &[Vec3]) -> Vec<Vec3> {
        let m = self.to_matrix();
        pts.iter().map(|&p| mat3_mul_vec(&m, p)).collect()
    }

    /// Like [`Self::rotate_points`], but reuses an existing buffer (no
    /// allocations once `out` has been sized; ideal for every frame).
    pub fn rotate_points_into(self, pts: &[Vec3], out: &mut Vec<Vec3>) {
        let m = self.to_matrix();
        out.clear();
        out.extend(pts.iter().map(|&p| mat3_mul_vec(&m, p)));
    }

    /// Converts to a 3x3 rotation matrix (`R = Ry(yaw) * Rx(pitch) * Rz(roll)`).
    #[must_use]
    pub fn to_matrix(self) -> Mat3 {
        let (sr, cr) = self.roll.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (sy, cy) = self.yaw.sin_cos();

        [
            [cy * cr + sy * sp * sr, -cy * sr + sy * sp * cr, sy * cp],
            [cp * sr, cp * cr, -sp],
            [-sy * cr + cy * sp * sr, sy * sr + cy * sp * cr, cy * cp],
        ]
    }

    /// Reconstructs normalized angles from a rotation matrix (using the same
    /// convention as [`Self::to_matrix`]).
    ///
    /// `cos(pitch)` is computed from matrix elements using `hypot`, rather
    /// than `asin` followed by `cos`, preserving precision near ±90°.
    /// At gimbal lock (pitch ≈ ±90°), roll and yaw are no longer independent:
    /// `yaw` is fixed at 0 and the rotation is represented through `roll`.
    #[must_use]
    pub fn from_matrix(m: Mat3) -> Self {
        const GIMBAL_EPS: f32 = 1e-4;

        let cos_pitch = m[1][0].hypot(m[1][1]);
        let pitch = (-m[1][2]).atan2(cos_pitch);

        if cos_pitch < GIMBAL_EPS {
            Self { roll: (-m[0][1]).atan2(m[0][0]), pitch, yaw: 0.0 }
        } else {
            Self {
                roll: m[1][0].atan2(m[1][1]),
                pitch,
                yaw: m[0][2].atan2(m[2][2]),
            }
        }
    }

    /// Returns the same rotation with normalized angles (`pitch` in
    /// `[-π/2, π/2]`, and `roll` and `yaw` in `[-π, π]`).
    #[must_use]
    pub fn normalized(self) -> Self {
        Self::from_matrix(self.to_matrix())
    }

    /// Composes two rotations: apply `self` first, then `other`
    /// (`other.rotate_point(self.rotate_point(p)) == self.compose(other).rotate_point(p)`).
    ///
    /// Computed using an exact matrix product, not by adding angles: 3D
    /// rotations do not commute.
    ///
    /// Returned angles are normalized (see the module documentation). Near
    /// gimbal lock (pitch ≈ ±90°), the rotation remains geometrically correct,
    /// but the angle may be redistributed between roll and yaw.
    #[must_use]
    pub fn compose(self, other: Rotation3D) -> Rotation3D {
        Self::from_matrix(mat3_mul(other.to_matrix(), self.to_matrix()))
    }

    /// Inverse rotation (`self.compose(self.inverse())` ≈ identity).
    #[must_use]
    pub fn inverse(self) -> Rotation3D {
        Self::from_matrix(mat3_transpose(self.to_matrix()))
    }

    /// Spherical interpolation between `self` (t = 0) and `other` (t = 1)
    /// along the shortest path, using quaternions.
    #[must_use]
    pub fn slerp(self, other: Rotation3D, t: f32) -> Rotation3D {
        crate::quat::Quat::from(self)
            .slerp(crate::quat::Quat::from(other), t)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    const PROBES: [Vec3; 4] = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, -2.0, 0.5],
    ];

    fn assert_vec3_close(a: Vec3, b: Vec3, eps: f32) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < eps, "a={a:?} b={b:?}");
        }
    }

    /// Two rotations are identical if they map the same points to the same
    /// locations, regardless of their angle representation.
    fn assert_same_effect(a: Rotation3D, b: Rotation3D, eps: f32) {
        for p in PROBES {
            assert_vec3_close(a.rotate_point(p), b.rotate_point(p), eps);
        }
    }

    fn grid(max: f32, n: i32) -> Vec<f32> {
        (-n..=n).map(|i| max * i as f32 / n as f32).collect()
    }

    struct Lcg(u32);
    impl Lcg {
        fn unit(&mut self) -> f32 {
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (self.0 >> 8) as f32 / (1u32 << 24) as f32
        }
        fn angle(&mut self) -> f32 {
            (self.unit() * 2.0 - 1.0) * PI
        }
    }

    #[test]
    fn identity_rotation_is_noop() {
        let p = [1.0, 2.0, 3.0];
        assert_vec3_close(Rotation3D::IDENTITY.rotate_point(p), p, 1e-6);
    }

    #[test]
    fn compose_matches_sequential_application() {
        let a = Rotation3D::new(0.3, 1.1, -0.7);
        let b = Rotation3D::new(-0.9, 0.4, 1.6);
        let p = [1.0, -2.0, 0.5];
        let sequential = b.rotate_point(a.rotate_point(p));
        let composed = a.compose(b).rotate_point(p);
        assert_vec3_close(sequential, composed, 1e-4);
    }

    #[test]
    fn naive_euler_addition_is_wrong() {
        // Two 90° rotations around different axes: adding the angles does not
        // produce the same result as composing the matrices.
        let a = Rotation3D::new(0.0, FRAC_PI_2, 0.0);
        let b = Rotation3D::new(FRAC_PI_2, 0.0, 0.0);
        let p = [0.0, 0.0, 1.0];

        let naive_sum =
            Rotation3D::new(a.roll + b.roll, a.pitch + b.pitch, a.yaw + b.yaw).rotate_point(p);
        let correct = a.compose(b).rotate_point(p);

        let diff: f32 = (0..3).map(|i| (naive_sum[i] - correct[i]).abs()).sum();
        assert!(diff > 1e-3, "the two approaches should differ here");
    }

    #[test]
    fn compose_with_identity_preserves_effect() {
        // Compare the effect rather than the angles: an out-of-range angle
        // (e.g. roll = 4.0) is returned in normalized form.
        for r in [
            Rotation3D::new(0.4, -0.2, 1.1),
            Rotation3D::new(4.0, 0.3, -5.0),
            Rotation3D::new(0.1, 2.1, 0.2),
        ] {
            assert_same_effect(r.compose(Rotation3D::IDENTITY), r, 1e-4);
            assert_same_effect(Rotation3D::IDENTITY.compose(r), r, 1e-4);
        }
    }

    #[test]
    fn matrix_roundtrip_preserves_effect() {
        for &roll in &grid(3.0, 4) {
            for &pitch in &grid(3.0, 4) {
                for &yaw in &grid(3.0, 4) {
                    let r = Rotation3D::new(roll, pitch, yaw);
                    let back = Rotation3D::from_matrix(r.to_matrix());
                    assert_same_effect(r, back, 1e-4);
                    assert!(back.pitch.abs() <= FRAC_PI_2 + 1e-6);
                    assert!(back.roll.abs() <= PI + 1e-6 && back.yaw.abs() <= PI + 1e-6);
                }
            }
        }
    }

    #[test]
    fn matrix_roundtrip_returns_same_angles_in_range() {
        // Within the normalized range and away from gimbal lock, the angles are recovered.
        let r = Rotation3D::new(0.5, -0.8, 2.0);
        let back = Rotation3D::from_matrix(r.to_matrix());
        assert!((back.roll - r.roll).abs() < 1e-5);
        assert!((back.pitch - r.pitch).abs() < 1e-5);
        assert!((back.yaw - r.yaw).abs() < 1e-5);
    }

    #[test]
    fn gimbal_lock_compose_stays_geometrically_correct() {
        let others = [Rotation3D::IDENTITY, Rotation3D::new(0.7, -0.3, 1.9)];
        for pitch in [FRAC_PI_2, -FRAC_PI_2, FRAC_PI_2 - 1e-4, -(FRAC_PI_2 - 1e-4)] {
            for &roll in &grid(3.0, 3) {
                for &yaw in &grid(3.0, 3) {
                    let a = Rotation3D::new(roll, pitch, yaw);
                    for b in others {
                        let composed = a.compose(b);
                        for p in PROBES {
                            let seq = b.rotate_point(a.rotate_point(p));
                            assert_vec3_close(composed.rotate_point(p), seq, 1e-2);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn compose_handles_out_of_range_pitch() {
        let a = Rotation3D::new(4.0, 2.1, -5.0);
        let b = Rotation3D::new(0.3, -2.5, 1.0);
        for p in PROBES {
            let seq = b.rotate_point(a.rotate_point(p));
            assert_vec3_close(a.compose(b).rotate_point(p), seq, 1e-4);
        }
    }

    #[test]
    fn rotation_preserves_length() {
        let p = [1.0, -2.0, 0.5];
        for &roll in &grid(3.0, 3) {
            for &pitch in &grid(3.0, 3) {
                for &yaw in &grid(3.0, 3) {
                    let out = Rotation3D::new(roll, pitch, yaw).rotate_point(p);
                    assert!((length(out) - length(p)).abs() < 1e-4);
                }
            }
        }
    }

    #[test]
    fn inverse_cancels_rotation() {
        for &roll in &grid(3.0, 2) {
            for &pitch in &grid(3.0, 2) {
                for &yaw in &grid(3.0, 2) {
                    let r = Rotation3D::new(roll, pitch, yaw);
                    assert_same_effect(r.compose(r.inverse()), Rotation3D::IDENTITY, 1e-3);
                }
            }
        }
    }

    #[test]
    fn rotate_points_matches_rotate_point() {
        let r = Rotation3D::new(0.3, 1.1, -0.7);
        let batch = r.rotate_points(&PROBES);
        let mut buf = Vec::new();
        r.rotate_points_into(&PROBES, &mut buf);
        for (i, p) in PROBES.iter().enumerate() {
            assert_vec3_close(batch[i], r.rotate_point(*p), 1e-6);
            assert_eq!(buf[i], batch[i]);
        }
    }

    #[test]
    fn random_compose_matches_sequential() {
        let mut rng = Lcg(0xC0FFEE);
        for _ in 0..1000 {
            let a = Rotation3D::new(rng.angle(), rng.angle() / 2.0, rng.angle());
            let b = Rotation3D::new(rng.angle(), rng.angle() / 2.0, rng.angle());
            for p in PROBES {
                let seq = b.rotate_point(a.rotate_point(p));
                assert_vec3_close(a.compose(b).rotate_point(p), seq, 2e-3);
            }
        }
    }
}