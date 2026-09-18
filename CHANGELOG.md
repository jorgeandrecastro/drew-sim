# Changelog

Toutes les modifications notables de ce projet sont documentées dans ce fichier.

Le format s'inspire de [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet suit [Semantic Versioning](https://semver.org/lang/fr/).



# Changelog

## [0.4.0]  2026-09-18

### Added
- `Quat` (quaternions) avec `from_euler`, `to_euler`, `from_axis_angle`, `compose`, `inverse`, `conjugate`, `normalize`, `slerp`, `to_matrix`, `rotate_point`, produit de Hamilton (`*`) et `From` dans les deux sens avec `Rotation3D`
- `Rotation3D::{inverse, slerp, normalized, to_matrix, from_matrix, rotate_points, rotate_points_into}`
- `math::{Mat3, IDENTITY_MAT3, mat3_mul, mat3_mul_vec, mat3_transpose}`
- `Mesh3D::edges()` et `Mesh3D::invalidate_edges()` (arêtes uniques, mises en cache)
- `GridStyle::with_*` (constructeurs chaînables)
- `OrbitCamera::reset()`
- `TrailBuffer::{len, is_empty, capacity}`
- Documentation des conventions de repère et d'angles ; `#![warn(missing_docs)]`
- Tests : aller-retour matrice, gimbal lock, conservation de la norme, inverse, composition sur angles hors plage, tirages pseudo-aléatoires

### Changed
- **BREAKING** : `Rotation3D::rotate_point` et `compose` prennent `self` au lieu de `&self`
- **BREAKING** : `Mesh3D` a un champ privé (cache d'arêtes) ; construire via `from_raw` / `unit_cube` / `from_stl_file`
- **BREAKING** : `GridStyle` et `MeshError` sont `#[non_exhaustive]`
- `Rotation3D::compose` : composition par produit de matrices (exacte), angles renvoyés normalisés
- `Rotation3D::from_matrix` : `cos(pitch)` calculé via `hypot`, précis près de ±90°
- `Rotation3D::new` et `TelemetryState::new` sont `const fn`
- `draw_mesh` : sommets tournés/projetés une seule fois, arêtes partagées dessinées une seule fois, dessin groupé ; `draw_trail` et `draw_chart` émettent une seule polyligne
- `TrailBuffer::push` : amorti O(1) (était O(n))
- `from_stl_file` valide les indices des triangles

### Fixed
- `TrailBuffer::new(0)` faisait paniquer `push`
- `lib.rs` : déclaration de module tronquée, exemple de la doc mis à jour pour `App::ui` (eframe 0.35)
- Coquilles du README

## [0.3.0] - 2026-08-27

### ⚠️ Changement de comportement à noter

`Rotation3D::compose` renvoyait un résultat **incorrect** dans la plupart des
cas (additionner naïvement les angles roll/pitch/yaw ne correspond pas à une
composition de rotations 3D valide, ces rotations ne commutent pas). C'est
corrigé dans cette version, mais si votre code appelait déjà `compose` et
compensait l'erreur ailleurs, les valeurs numériques produites vont changer.
La signature ne change pas, donc ce n'est pas un breaking change au sens de
semver, mais le comportement, lui, change réellement  à revalider si vous
dépendiez de l'ancien résultat.

### Fixed
- `Rotation3D::compose` passe maintenant par une composition matricielle
  exacte (matrices de rotation 3x3 en interne) au lieu d'additionner les
  angles d'Euler. Le résultat correspond désormais bien à l'application
  séquentielle des deux rotations.
- Gestion explicite du cas dégénéré de gimbal lock (`pitch ≈ ±90°`) lors de
  la reconstruction des angles, pour éviter un résultat incohérent.

### Added
- `math::dot(a, b)` : produit scalaire.
- `math::cross(a, b)` : produit vectoriel.
- `math::length(a)` : norme d'un `Vec3`.
- `math::normalize(a)` : normalisation d'un `Vec3` (sûre en cas de vecteur
  quasi nul).
- Tests pour `compose` : cohérence avec l'application séquentielle des
  rotations, cas identité, et régression documentant explicitement l'ancien
  bug d'addition d'angles.

### Changed
- Toutes les fonctions de `math.rs` sont maintenant annotées `#[must_use]`.
- README : le tableau `Rotation3D` ne décrit plus `compose` comme une
  "approximation par angles d'Euler".

## [0.2.0] - date inconnue

- Version initiale documentée (voir README pour l'API complète : `Engine3D`,
  `Render3D`, `Mesh3D`, `TelemetryState`/`TrailBuffer`, `Rotation3D`).