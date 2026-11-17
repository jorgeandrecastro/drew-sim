# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [0.4.1]

- Migrated the documentation, README, and changelog to English.

## [0.4.0]  2026-09-18

### Added
- `Quat` (quaternions) with `from_euler`, `to_euler`, `from_axis_angle`, `compose`, `inverse`, `conjugate`, `normalize`, `slerp`, `to_matrix`, `rotate_point`, the Hamilton product (`*`), and bidirectional `From` conversions with `Rotation3D`
- `Rotation3D::{inverse, slerp, normalized, to_matrix, from_matrix, rotate_points, rotate_points_into}`
- `math::{Mat3, IDENTITY_MAT3, mat3_mul, mat3_mul_vec, mat3_transpose}`
- `Mesh3D::edges()` and `Mesh3D::invalidate_edges()` (cached unique edges)
- `GridStyle::with_*` (chainable builders)
- `OrbitCamera::reset()`
- `TrailBuffer::{len, is_empty, capacity}`
- Documentation of coordinate-frame and angle conventions; `#![warn(missing_docs)]`
- Tests: matrix round trips, gimbal lock, norm preservation, inverse, composition with out-of-range angles, pseudorandom samples

### Changed
- **BREAKING**: `Rotation3D::rotate_point` and `compose` take `self` instead of `&self`
- **BREAKING**: `Mesh3D` has a private field (edge cache); construct it using `from_raw` / `unit_cube` / `from_stl_file`
- **BREAKING**: `GridStyle` and `MeshError` are `#[non_exhaustive]`
- `Rotation3D::compose`: exact matrix-product composition; returned angles are normalized
- `Rotation3D::from_matrix`: `cos(pitch)` is computed using `hypot`, retaining precision near ±90°
- `Rotation3D::new` and `TelemetryState::new` are `const fn`
- `draw_mesh`: vertices are rotated/projected once, shared edges are drawn once, and drawing is batched; `draw_trail` and `draw_chart` each emit a single polyline
- `TrailBuffer::push`: amortized O(1) (previously O(n))
- `from_stl_file` validates triangle indices

### Fixed
- `TrailBuffer::new(0)` caused `push` to panic
- `lib.rs`: fixed a truncated module declaration; updated the documentation example to use `App::ui` (eframe 0.35)
- README typos

## [0.3.0] - 2026-08-27

### ⚠️ Behavior change to note

`Rotation3D::compose` returned an **incorrect** result in most cases (naively
adding roll/pitch/yaw angles does not produce a valid composition of 3D
rotations, since these rotations do not commute). This is fixed in this
release, but if your code already called `compose` and compensated for the
error elsewhere, the resulting numeric values will change. The signature is
unchanged, so this is not a breaking change in the SemVer sense, but the
behavior does change; revalidate your code if it depended on the old result.

### Fixed
- `Rotation3D::compose` now uses exact matrix composition (3x3 rotation
  matrices internally) instead of adding Euler angles. The result now
  correctly matches the sequential application of the two rotations.
- Explicit handling of the degenerate gimbal-lock case (`pitch ≈ ±90°`) when
  reconstructing angles, to avoid inconsistent results.

### Added
- `math::dot(a, b)`: dot product.
- `math::cross(a, b)`: cross product.
- `math::length(a)`: magnitude of a `Vec3`.
- `math::normalize(a)`: normalize a `Vec3` (safe for near-zero vectors).
- Tests for `compose`: consistency with sequential rotation application,
  identity cases, and a regression test explicitly documenting the previous
  angle-addition bug.

### Changed
- All functions in `math.rs` are now annotated with `#[must_use]`.
- README: the `Rotation3D` table no longer describes `compose` as an
  "approximation using Euler angles."

## [0.2.0] - date unknown

- Initial documented version (see the README for the full API: `Engine3D`,
  `Render3D`, `Mesh3D`, `TelemetryState`/`TrailBuffer`, `Rotation3D`).