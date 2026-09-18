//! Primitives mathématiques : vecteurs 3D et rotations d'Euler.
//!
//! # Conventions
//!
//! - Repère **main droite**, angles en **radians**, positifs dans le sens
//!   trigonométrique (règle de la main droite autour de l'axe).
//! - `roll` tourne autour de **Z**, `pitch` autour de **X**, `yaw` autour de **Y**.
//! - Ordre d'application : roll, puis pitch, puis yaw, soit
//!   `R = Ry(yaw) * Rx(pitch) * Rz(roll)`.
//! - Y est l'axe vertical de la scène ; Z, axe de roulis, est l'« avant »
//!   du véhicule.
//! - Les angles renvoyés par [`Rotation3D::compose`], [`Rotation3D::inverse`]
//!   et [`Rotation3D::from_matrix`] sont **normalisés** : `pitch` dans
//!   `[-π/2, π/2]`, `roll` et `yaw` dans `[-π, π]`. Le triplet peut donc
//!   différer de celui d'entrée tout en représentant la même rotation.

/// Vecteur / point 3D (x, y, z).
pub type Vec3 = [f32; 3];

/// Matrice 3x3, stockée ligne par ligne.
pub type Mat3 = [[f32; 3]; 3];

/// Matrice identité 3x3.
pub const IDENTITY_MAT3: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Addition de deux vecteurs.
#[must_use]
#[inline]
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Soustraction de deux vecteurs.
#[must_use]
#[inline]
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Mise à l'échelle d'un vecteur par un scalaire.
#[must_use]
#[inline]
pub fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Produit scalaire.
#[must_use]
#[inline]
pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Produit vectoriel.
#[must_use]
#[inline]
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Norme (longueur) du vecteur.
#[must_use]
#[inline]
pub fn length(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}

/// Normalise le vecteur. Renvoie le vecteur d'origine si sa longueur
/// est trop proche de zéro (évite une division par zéro).
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

/// Produit de deux matrices 3x3 (`a * b`).
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

/// Produit matrice x vecteur (`m * v`).
#[must_use]
#[inline]
pub fn mat3_mul_vec(m: &Mat3, v: Vec3) -> Vec3 {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}

/// Transposée d'une matrice 3x3 (= inverse pour une matrice de rotation).
#[must_use]
pub fn mat3_transpose(m: Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Rotation exprimée en angles d'Euler (radians).
///
/// Voir les conventions dans la documentation du module.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rotation3D {
    /// Roulis : rotation autour de Z (radians).
    pub roll: f32,
    /// Tangage : rotation autour de X (radians).
    pub pitch: f32,
    /// Lacet : rotation autour de Y (radians).
    pub yaw: f32,
}

impl Rotation3D {
    /// Rotation nulle.
    pub const IDENTITY: Self = Self { roll: 0.0, pitch: 0.0, yaw: 0.0 };

    /// Construit une rotation à partir de `roll`, `pitch`, `yaw` (radians).
    #[must_use]
    #[inline]
    pub const fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self { roll, pitch, yaw }
    }

    /// Applique la rotation (roll, puis pitch, puis yaw) à un point 3D.
    ///
    /// Pour beaucoup de points, préférer [`Self::rotate_points`] qui ne
    /// calcule la matrice qu'une fois.
    #[must_use]
    #[inline]
    pub fn rotate_point(self, p: Vec3) -> Vec3 {
        mat3_mul_vec(&self.to_matrix(), p)
    }

    /// Applique la rotation à un lot de points (matrice calculée une seule fois).
    #[must_use]
    pub fn rotate_points(self, pts: &[Vec3]) -> Vec<Vec3> {
        let m = self.to_matrix();
        pts.iter().map(|&p| mat3_mul_vec(&m, p)).collect()
    }

    /// Comme [`Self::rotate_points`], mais réutilise un buffer existant
    /// (aucune allocation une fois `out` dimensionné : idéal à chaque frame).
    pub fn rotate_points_into(self, pts: &[Vec3], out: &mut Vec<Vec3>) {
        let m = self.to_matrix();
        out.clear();
        out.extend(pts.iter().map(|&p| mat3_mul_vec(&m, p)));
    }

    /// Convertit en matrice de rotation 3x3 (`R = Ry(yaw) * Rx(pitch) * Rz(roll)`).
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

    /// Reconstruit des angles normalisés à partir d'une matrice de rotation
    /// (même convention que [`Self::to_matrix`]).
    ///
    /// `cos(pitch)` est obtenu à partir des éléments de la matrice (`hypot`)
    /// et non via `asin` puis `cos`, ce qui reste précis près de ±90°.
    /// Au gimbal lock (pitch ≈ ±90°), roll et yaw ne sont plus
    /// indépendants : `yaw` est figé à 0 et tout est reporté sur `roll`.
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

    /// Renvoie la même rotation avec des angles normalisés
    /// (`pitch` dans `[-π/2, π/2]`, `roll` et `yaw` dans `[-π, π]`).
    #[must_use]
    pub fn normalized(self) -> Self {
        Self::from_matrix(self.to_matrix())
    }

    /// Compose deux rotations : appliquer `self` d'abord, puis `other`
    /// (`other.rotate_point(self.rotate_point(p)) == self.compose(other).rotate_point(p)`).
    ///
    /// Calculé par produit de matrices (exact), pas par somme d'angles :
    /// les rotations 3D ne commutent pas.
    ///
    /// Les angles renvoyés sont normalisés (voir la doc du module).
    /// Près du gimbal lock (pitch ≈ ±90°), la rotation reste
    /// géométriquement correcte mais l'angle peut être redistribué entre
    /// roll et yaw.
    #[must_use]
    pub fn compose(self, other: Rotation3D) -> Rotation3D {
        Self::from_matrix(mat3_mul(other.to_matrix(), self.to_matrix()))
    }

    /// Rotation inverse (`self.compose(self.inverse())` ≈ identité).
    #[must_use]
    pub fn inverse(self) -> Rotation3D {
        Self::from_matrix(mat3_transpose(self.to_matrix()))
    }

    /// Interpolation sphérique entre `self` (t = 0) et `other` (t = 1),
    /// par le chemin le plus court, via quaternions.
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

    /// Deux rotations sont identiques si elles envoient les mêmes points
    /// aux mêmes endroits (indépendant de la représentation en angles).
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
        // Deux rotations de 90° sur des axes différents : additionner les
        // angles ne donne pas le même résultat que composer les matrices.
        let a = Rotation3D::new(0.0, FRAC_PI_2, 0.0);
        let b = Rotation3D::new(FRAC_PI_2, 0.0, 0.0);
        let p = [0.0, 0.0, 1.0];

        let naive_sum =
            Rotation3D::new(a.roll + b.roll, a.pitch + b.pitch, a.yaw + b.yaw).rotate_point(p);
        let correct = a.compose(b).rotate_point(p);

        let diff: f32 = (0..3).map(|i| (naive_sum[i] - correct[i]).abs()).sum();
        assert!(diff > 1e-3, "les deux approches devraient diverger ici");
    }

    #[test]
    fn compose_with_identity_preserves_effect() {
        // On compare l'effet et non les angles : un angle hors plage
        // (ex. roll = 4.0) ressort normalisé.
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
        // Dans la plage normalisée et loin du gimbal lock, on retrouve les angles.
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