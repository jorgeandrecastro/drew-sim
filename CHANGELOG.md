# Changelog

Toutes les modifications notables de ce projet sont documentées dans ce fichier.

Le format s'inspire de [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet suit [Semantic Versioning](https://semver.org/lang/fr/).

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