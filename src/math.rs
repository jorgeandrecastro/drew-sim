//! Primitives mathématiques : vecteurs 3D et rotations Euler.

/// Vecteur / point 3D (x, y, z).
pub type Vec3 = [f32; 3];

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

/// Matrice de rotation 3x3, stockée ligne par ligne.
/// Usage interne uniquement : sert de représentation intermédiaire
/// pour composer correctement des rotations (voir `Rotation3D::compose`).
type Mat3 = [[f32; 3]; 3];

const IDENTITY_MAT3: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn mat3_mul(a: Mat3, b: Mat3) -> Mat3 {
    let mut out = IDENTITY_MAT3;
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// Représente les 3 angles de rotation en radians (Roulis, Tangage, Lacet).
///
/// Convention : Roll autour de Z, Pitch autour de X, Yaw autour de Y,
/// appliqués dans cet ordre (Roll puis Pitch puis Yaw), soit
/// `R = Ry(yaw) * Rx(pitch) * Rz(roll)`.
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

    /// Convertit en matrice de rotation 3x3 (`R = Ry(yaw) * Rx(pitch) * Rz(roll)`).
    fn to_matrix(self) -> Mat3 {
        let (sr, cr) = self.roll.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (sy, cy) = self.yaw.sin_cos();

        [
            [cy * cr + sy * sp * sr, -cy * sr + sy * sp * cr, sy * cp],
            [cp * sr, cp * cr, -sp],
            [-sy * cr + cy * sp * sr, sy * sr + cy * sp * cr, cy * cp],
        ]
    }

    /// Reconstruit les angles (roll, pitch, yaw) à partir d'une matrice
    /// de rotation dans la même convention que `to_matrix`.
    /// Gère le cas dégénéré (gimbal lock, pitch ≈ ±90°) en figeant yaw à 0.
    fn from_matrix(m: Mat3) -> Self {
        let sin_pitch = (-m[1][2]).clamp(-1.0, 1.0);
        let pitch = sin_pitch.asin();
        let cos_pitch = pitch.cos();

        const GIMBAL_EPS: f32 = 1e-5;
        if cos_pitch.abs() < GIMBAL_EPS {
            // Gimbal lock : roll et yaw ne sont plus indépendants.
            // On fixe yaw = 0 et on reporte toute la rotation restante sur roll.
            let roll = (-m[0][1]).atan2(m[0][0]);
            Self { roll, pitch, yaw: 0.0 }
        } else {
            let roll = m[1][0].atan2(m[1][1]);
            let yaw = m[0][2].atan2(m[2][2]);
            Self { roll, pitch, yaw }
        }
    }

    /// Compose deux rotations : le résultat équivaut à appliquer `self`
    /// d'abord, puis `other` (`other.rotate_point(self.rotate_point(p))`
    /// == `self.compose(other).rotate_point(p)`).
    ///
    /// Implémenté via une multiplication de matrices de rotation (exacte),
    /// pas par une simple somme d'angles d'Euler — les rotations 3D ne
    /// commutent pas, donc additionner roll/pitch/yaw indépendamment donne
    /// un résultat faux dès que les rotations combinées ne sont pas de
    /// petits angles autour du même axe.
    ///
    /// Note : proche du gimbal lock (pitch ≈ ±90°), roll et yaw ne sont
    /// plus discernables ; le résultat reste géométriquement correct mais
    /// peut redistribuer l'angle entre roll et yaw de façon inattendue.
    pub fn compose(&self, other: Rotation3D) -> Rotation3D {
        let combined = mat3_mul(other.to_matrix(), self.to_matrix());
        Self::from_matrix(combined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vec3_close(a: Vec3, b: Vec3, eps: f32) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < eps, "a={a:?} b={b:?}");
        }
    }

    #[test]
    fn identity_rotation_is_noop() {
        let p = [1.0, 2.0, 3.0];
        let r = Rotation3D::IDENTITY;
        let out = r.rotate_point(p);
        assert_vec3_close(out, p, 1e-6);
    }

    #[test]
    fn compose_matches_sequential_application() {
        // La propriété qu'on veut garantir : composer puis appliquer
        // équivaut à appliquer les deux rotations l'une après l'autre,
        // quel que soit l'angle (contrairement à l'ancienne implémentation
        // qui additionnait juste les angles).
        let a = Rotation3D::new(0.3, 1.1, -0.7);
        let b = Rotation3D::new(-0.9, 0.4, 1.6);
        let p = [1.0, -2.0, 0.5];

        let sequential = b.rotate_point(a.rotate_point(p));
        let composed = a.compose(b).rotate_point(p);

        assert_vec3_close(sequential, composed, 1e-4);
    }

    #[test]
    fn naive_euler_addition_would_be_wrong() {
        // Documente pourquoi l'ancienne implémentation était fausse :
        // pour deux rotations à 90° sur des axes différents, additionner
        // les angles ne donne pas le même résultat que composer les
        // matrices.
        let a = Rotation3D::new(0.0, std::f32::consts::FRAC_PI_2, 0.0); // pitch 90°
        let b = Rotation3D::new(std::f32::consts::FRAC_PI_2, 0.0, 0.0); // roll 90°
        let p = [0.0, 0.0, 1.0];

        let naive_sum = Rotation3D::new(
            a.roll + b.roll,
            a.pitch + b.pitch,
            a.yaw + b.yaw,
        )
        .rotate_point(p);

        let correct = a.compose(b).rotate_point(p);

        let diff: f32 = (0..3).map(|i| (naive_sum[i] - correct[i]).abs()).sum();
        assert!(diff > 1e-3, "les deux approches devraient diverger ici");
    }

    #[test]
    fn compose_with_identity_is_noop() {
        let r = Rotation3D::new(0.4, -0.2, 1.1);
        let composed = r.compose(Rotation3D::IDENTITY);
        assert!((composed.roll - r.roll).abs() < 1e-5);
        assert!((composed.pitch - r.pitch).abs() < 1e-5);
        assert!((composed.yaw - r.yaw).abs() < 1e-5);
    }
}