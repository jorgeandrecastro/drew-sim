# drew-sim

[![Crates.io](https://img.shields.io/crates/v/drew-sim.svg)](https://crates.io/crates/drew-sim)
[![Downloads](https://img.shields.io/crates/d/drew-sim.svg)](https://crates.io/crates/drew-sim)
[![docs.rs](https://docs.rs/drew-sim/badge.svg)](https://docs.rs/drew-sim)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)

An immediate-mode 3D rendering library for [`eframe`]/[`egui`] (0.35+),
designed for real-time telemetry visualization: drones, robots, and
orientation sensors.

## Installation

```toml
[dependencies]
drew-sim = "0.5"
```

The `stl` feature (for loading `.stl` files) is enabled by default.
To disable it:

```toml
drew-sim = { version = "0.5", default-features = false }
```

## Mouse controls (3D rendering area)

| Action                                   | Effect                  |
|------------------------------------------|-------------------------|
| Left-click and drag                      | Orbit and tilt the camera |
| Scroll wheel                             | Zoom                    |
| Right-click **or** middle-click and drag | Pan                     |

These controls are handled automatically by `Engine3D::render`; no application
code is needed. There is no on-screen camera widget: all camera interaction
is done with the mouse. Call `engine.camera.reset()` to restore the camera's
initial position.

## Conventions

- **Right-handed** coordinate system; angles are in **radians** and positive
  in the counterclockwise direction (the right-hand rule around the axis).
- `roll` rotates around **Z**, `pitch` around **X**, and `yaw` around **Y**,
  applied in this order: `R = Ry(yaw) * Rx(pitch) * Rz(roll)`.
- Y is the scene's vertical axis; Z (the roll axis) points toward the
  vehicle's front.
- Angles returned by `compose`, `inverse`, `normalized`, and `from_matrix` are
  **normalized**: `pitch` is in [-π/2, π/2], and `roll` and `yaw` are in
  [-π, π]. The returned triplet may differ from the input while representing
  the same rotation; compare the effect on points rather than the angles.
- Quaternions (`Quat`) use the same convention:
  `Quat::from_euler(r)` is equivalent to `Ry(yaw) * Rx(pitch) * Rz(roll)`.
- The camera orbits fully: **azimuth** (`orbit`) around the vertical axis and
  **elevation** (`elevation`, clamped just below ±90°). The projection is
  **perspective** by default; set `engine.camera.perspective = false` for an
  orthographic projection.

## Useful functionality

### `Engine3D`: high-level engine (`src/engine.rs`)

| Function | Role |
|---|---|
| `Engine3D::default()` | Perspective camera with `orbit=0.5`, `elevation=0.35`, `zoom=1.0`, and the default grid |
| `engine.render(ui, \|r\| { ... })` | Allocates the drawing area, applies mouse input to the camera, draws the 3D ground grid (plane y = 0), then calls the closure with a ready-to-use `Render3D` |
| `engine.camera` | Direct access to the `OrbitCamera` (e.g. to set an initial angle) |
| `engine.camera.reset()` | Resets the orbit, elevation, zoom, and pan to their default values (the projection mode is kept) |
| `engine.camera.perspective` | `true`: perspective (default); `false`: orthographic |
| `engine.camera.distance` | Camera distance from the origin, in world units (perspective only) |
| `engine.grid` | `GridStyle`: ground-grid colors and spacing |
| `GridStyle::default().with_step_px(30.0).with_half_lines(20)` | Customizes the grid (`with_line_color`, `with_ground_color`, `with_background_color`, `with_step_px`, `with_half_lines`). `GridStyle` is `#[non_exhaustive]`; start with `default()` |

`OrbitCamera` is `#[non_exhaustive]`: start with `OrbitCamera::default()` and
then modify its public fields (`orbit`, `elevation`, `zoom`, `pan_offset`,
`perspective`, `distance`).

### `Render3D`: drawing primitives (`src/render.rs`), used inside the `render(...)` closure

| Function | Role |
|---|---|
| `r.world_to_screen(pos)` | Projects a 3D point to the screen (points behind the camera are clamped to the near plane) |
| `r.world_to_view(pos)` | World point to view space (x right, y up, z = depth) |
| `r.project_view(v)` | Projects a view-space point; returns `None` if it is behind the camera |
| `r.draw_line(a, b, stroke)` | 3D line segment between two world points (clipped against the near plane) |
| `r.draw_mesh(&mesh, position, rotation, color)` | Wireframe mesh (each vertex is projected once, and shared edges are drawn once) |
| `r.draw_mesh_with_stroke(&mesh, position, rotation, stroke)` | Same, with a full `egui::Stroke` (including line width) |
| `r.draw_mesh_solid(&mesh, position, rotation, color)` | Flat-shaded solid faces (painter's algorithm: triangles sorted back to front, two-sided lighting). Without a z-buffer, strongly concave or interpenetrating objects may show sorting artifacts |
| `r.draw_axes(position, rotation, length)` | XYZ axes (red = X, green = Y, blue = Z) |
| `r.draw_trail(&points, color)` | Polyline connecting a history of positions (parts behind the camera are cut) |
| `r.draw_chart(rect, &values, min, max, color)` | 2D chart (value curve), independent of the 3D camera, for displaying a sensor overlay |

### `Mesh3D` (`src/mesh.rs`)

| Function | Role |
|---|---|
| `Mesh3D::from_stl_file(path)` | Loads an `.stl` file (requires the `stl` feature) |
| `Mesh3D::unit_cube(size)` | Creates a cube without external dependencies, useful for a quick test |
| `Mesh3D::from_raw(vertices, triangles)` | Builds a mesh manually, validating indices |
| `mesh.triangle_count()` | Number of triangles |
| `mesh.edges()` | Unique edges (computed once, then cached) |
| `mesh.invalidate_edges()` | Call after manually modifying `mesh.triangles` |

### `TelemetryState` / `TrailBuffer` (`src/telemetry.rs`)

| Function | Role |
|---|---|
| `TelemetryState { position, rotation }` | Source-agnostic data contract (physical sensor, simulation, or log playback) |
| `TrailBuffer::new(capacity)` | History of the last `capacity` positions (avoids unbounded memory growth in long sessions; amortized O(1) `push`) |
| `trail.push(pos)` / `trail.as_slice()` / `trail.clear()` | Add to and read from the buffer |
| `trail.len()` / `trail.is_empty()` / `trail.capacity()` | Current length and capacity |

### Vectors (`src/math.rs`)

| Function | Role |
|---|---|
| `add(a, b)` / `sub(a, b)` | Add / subtract two `Vec3` values |
| `scale(a, s)` | Scale a `Vec3` by a scalar |
| `dot(a, b)` | Dot product |
| `cross(a, b)` | Cross product |
| `length(a)` | Vector magnitude |
| `normalize(a)` | Normalize a vector (returns `a` unchanged if it is near zero) |
| `mat3_mul(a, b)` / `mat3_mul_vec(&m, v)` / `mat3_transpose(m)` | 3x3 matrix product, matrix-vector product, and transpose |

### Euler rotations (`Rotation3D`, `src/math.rs`)

| Function | Role |
|---|---|
| `Rotation3D::new(roll, pitch, yaw)` | Angles in radians (`const fn`) |
| `Rotation3D::IDENTITY` | Zero rotation |
| `rotation.rotate_point(p)` | Applies the rotation to a point |
| `rotation.rotate_points(&pts)` / `rotate_points_into(&pts, &mut out)` | Rotates a batch of points, computing the matrix once (the latter reuses a buffer) |
| `rotation.compose(other)` | Applies `self`, then `other` (exact matrix product). Near gimbal lock (pitch ≈ ±90°), the result remains geometrically correct, but roll and yaw may be redistributed (yaw is fixed at 0) |
| `rotation.inverse()` | Inverse rotation |
| `rotation.slerp(other, t)` | Spherical interpolation along the shortest path (for log playback) |
| `rotation.normalized()` | Same rotation, with angles mapped to the normalized range |
| `rotation.to_matrix()` / `Rotation3D::from_matrix(m)` | Convert between angles and a 3x3 matrix |

### Quaternions (`Quat`, `src/quat.rs`)

Native format for IMUs and autopilots (PX4, ArduPilot, BNO055, etc.), with no
gimbal lock.

| Function | Role |
|---|---|
| `Quat::IDENTITY` / `Quat::new(w, x, y, z)` | Zero rotation / construct from components |
| `Quat::from_axis_angle(axis, angle)` | Rotation around an axis (the axis is normalized) |
| `Quat::from_euler(r)` / `q.to_euler()` | Convert from / to `Rotation3D` (also available through `From`) |
| `q.compose(other)` | Applies `q`, then `other`, with the same semantics as `Rotation3D::compose` |
| `a * b` | Hamilton product: applies `b` first, then `a` (like a matrix product) |
| `q.inverse()` / `q.conjugate()` | Inverse / conjugate |
| `q.normalize()` | Renormalizes (do this regularly when integrating angular velocities to avoid drift) |
| `q.slerp(other, t)` | Spherical interpolation along the shortest path |
| `q.rotate_point(p)` / `q.to_matrix()` | Apply to a point / convert to a 3x3 matrix |

## Minimal example

Starting with egui/eframe 0.35, `App::update(ctx, frame)` was replaced by
`App::ui(ui, frame)`: the app receives a `&mut egui::Ui` instead of an
`&egui::Context`, and panels are drawn with `.show(ui, ...)` instead of
`.show(ctx, ...)`. To access the `Context` (e.g. to call `request_repaint`),
use `ui.ctx()`.

```rust
use drew_sim::{Engine3D, Mesh3D, TelemetryState};
use drew_sim::egui;

struct App {
    engine: Engine3D,
    mesh: Mesh3D,
    state: TelemetryState,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            self.engine.render(ui, |r| {
                r.draw_mesh_solid(&self.mesh, self.state.position, self.state.rotation, egui::Color32::LIGHT_BLUE);
                r.draw_axes(self.state.position, self.state.rotation, 1.5);
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "drew-sim minimal",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(App {
                engine: Engine3D::default(),
                mesh: Mesh3D::unit_cube(1.5),
                state: TelemetryState::default(),
            }))
        }),
    )
}
```

## Quaternion example (IMU)

```rust
use drew_sim::{Quat, Rotation3D, TelemetryState};

// Quaternion (w, x, y, z) provided by the sensor, normalized as a precaution.
let q = Quat::new(0.9239, 0.0, 0.3827, 0.0).normalize();
let state = TelemetryState::new([0.0, 1.0, 0.0], q.to_euler());
```

The `(w, x, y, z)` components provided by a sensor depend on its coordinate
frame (NED, ENU, etc.); convert them to the drew-sim coordinate frame before
using them (see [Conventions](#conventions)).

## Full example

A richer example (animated cube, control panel, solid/wireframe toggle,
perspective toggle, trail, and altitude chart) is available:

```bash
cargo run --example orbit_viewer --release
```

## Migration from 0.4 to 0.5

- `OrbitCamera` is `#[non_exhaustive]` and has new fields (`elevation`,
  `perspective`, `distance`): start with `OrbitCamera::default()` instead of a
  struct literal.
- `Render3D` has new public fields (`camera_elevation`, `perspective`,
  `camera_distance`). This only affects code that builds a `Render3D` manually;
  `Engine3D::render` fills them in for you.
- The default view is now **perspective** with a slight downward tilt. To get
  the previous flat look, set `engine.camera.perspective = false` and
  `engine.camera.elevation = 0.0`.
- The ground grid is now a real 3D grid on the y = 0 plane. `GridStyle::step_px`
  is converted to world units (`step_px / 50`).
- `world_to_screen` now takes the elevation and the projection mode into
  account.

## Roadmap

- [x] Solid-face rendering (flat shading, painter's algorithm)
- [x] Perspective camera with tilt (full orbit)
- [ ] True z-buffer (correct occlusion for concave or interpenetrating meshes)
- [ ] Import other mesh formats (OBJ, glTF)
- [ ] Sharp-edge (crease angle) filter for dense wireframes
- [ ] Pose / transform tree, URDF loading, and `JointState` input (articulated robots)
- [ ] Telemetry frame conversions (NED, ENU, ROS REP-103)
- [ ] Optional HUD showing mouse controls and camera state
- [ ] Optional `glam` / `mint` features (`Quat`/`Vec3` conversions)
- [ ] CI (fmt, clippy, tests, docs) and declared MSRV

## License

GPL-2.0-or-later; see [LICENSE](LICENSE).

Copyright (C) 2026 Jorge Andre Castro