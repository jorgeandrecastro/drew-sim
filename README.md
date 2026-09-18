# drew-sim

[![Crates.io](https://img.shields.io/crates/v/drew-sim.svg)](https://crates.io/crates/drew-sim)
[![Downloads](https://img.shields.io/crates/d/drew-sim.svg)](https://crates.io/crates/drew-sim)
[![docs.rs](https://docs.rs/drew-sim/badge.svg)](https://docs.rs/drew-sim)
[![License: GPL-2.0-or-later](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)

Librairie de rendu 3D immédiat pour applications [`eframe`]/[`egui`] (0.35+), pensée
pour la visualisation de télémétrie temps réel : drones, robots, capteurs
d'orientation.

## Installation

```toml
[dependencies]
drew-sim = "0.4"
```

La feature `stl` (chargement de fichiers `.stl`) est activée par défaut.
Pour s'en passer :

```toml
drew-sim = { version = "0.4", default-features = false }
```

## Contrôles souris (zone de rendu 3D)

| Action                                   | Effet            |
|------------------------------------------|------------------|
| Clic gauche + glisser                    | Orbite la caméra |
| Molette                                  | Zoom             |
| Clic droit **ou** clic molette + glisser | Panoramique      |

Ces contrôles sont gérés automatiquement par `Engine3D::render` : rien à
faire côté application. En contrepartie, rien n'est affiché à l'écran
(pas de widget de caméra visible) : tout se fait à la souris.
`engine.camera.reset()` remet la caméra dans sa position initiale.

## Conventions

- Repère **main droite**, angles en **radians**, positifs dans le sens
  trigonométrique (règle de la main droite autour de l'axe).
- `roll` tourne autour de **Z**, `pitch` autour de **X**, `yaw` autour de **Y**,
  appliqués dans cet ordre : `R = Ry(yaw) * Rx(pitch) * Rz(roll)`.
- Y est l'axe vertical de la scène ; Z (axe de roulis) est l'« avant » du
  véhicule.
- Les angles renvoyés par `compose`, `inverse`, `normalized` et `from_matrix`
  sont **normalisés** : `pitch` dans [-π/2, π/2], `roll` et `yaw` dans
  [-π, π]. Le triplet peut donc différer de celui d'entrée tout en
  représentant la même rotation ; comparer l'effet sur des points plutôt que
  les angles.
- Les quaternions (`Quat`) suivent la même convention :
  `Quat::from_euler(r)` équivaut à `Ry(yaw) * Rx(pitch) * Rz(roll)`.
- La projection est **orthographique** avec une orbite horizontale
  uniquement (pas de perspective, pas d'inclinaison de caméra).

## Fonctions utiles

### `Engine3D` : moteur haut niveau (`src/engine.rs`)

| Fonction | Rôle |
|---|---|
| `Engine3D::default()` | Caméra à `orbit=0.5`, `zoom=1.0`, grille par défaut |
| `engine.render(ui, \|r\| { ... })` | Alloue la zone de dessin, applique la souris à la caméra, dessine la grille de sol, puis appelle la closure avec un `Render3D` prêt à l'emploi |
| `engine.camera` | Accès direct à l'`OrbitCamera` (ex : forcer un angle au démarrage) |
| `engine.camera.reset()` | Remet orbite, zoom et panoramique à leurs valeurs par défaut |
| `engine.grid` | `GridStyle` : couleurs et espacement de la grille au sol |
| `GridStyle::default().with_step_px(30.0).with_half_lines(20)` | Personnalise la grille (`with_line_color`, `with_ground_color`, `with_background_color`, `with_step_px`, `with_half_lines`). `GridStyle` est `#[non_exhaustive]` : on part de `default()` |

### `Render3D` : primitives de dessin (`src/render.rs`), utilisées dans la closure de `render(...)`

| Fonction | Rôle |
|---|---|
| `r.world_to_screen(pos)` | Projette un point 3D vers l'écran (utile pour dessiner autre chose que les primitives ci-dessous) |
| `r.draw_mesh(&mesh, position, rotation, color)` | Maillage en fil de fer (sommets projetés une fois, arêtes partagées dessinées une fois) |
| `r.draw_mesh_with_stroke(&mesh, position, rotation, stroke)` | Idem, avec un `egui::Stroke` complet (épaisseur de trait) |
| `r.draw_axes(position, rotation, length)` | Repère XYZ (Rouge = X, Vert = Y, Bleu = Z) |
| `r.draw_trail(&points, color)` | Polyligne reliant un historique de positions |
| `r.draw_chart(rect, &values, min, max, color)` | Graphe 2D (courbe de valeurs), indépendant de la caméra 3D, pour afficher un capteur en overlay |

### `Mesh3D` (`src/mesh.rs`)

| Fonction | Rôle |
|---|---|
| `Mesh3D::from_stl_file(path)` | Charge un `.stl` (nécessite la feature `stl`) |
| `Mesh3D::unit_cube(size)` | Cube généré sans dépendance externe, pratique pour tester rapidement |
| `Mesh3D::from_raw(vertices, triangles)` | Construit un maillage à la main, avec validation des indices |
| `mesh.triangle_count()` | Nombre de triangles |
| `mesh.edges()` | Arêtes uniques (calculées une fois, puis en cache) |
| `mesh.invalidate_edges()` | À appeler après avoir modifié `mesh.triangles` à la main |

### `TelemetryState` / `TrailBuffer` (`src/telemetry.rs`)

| Fonction | Rôle |
|---|---|
| `TelemetryState { position, rotation }` | Contrat agnostique de la source (capteur réel, simulation, rejeu de log) |
| `TrailBuffer::new(capacity)` | Historique des `capacity` dernières positions (évite une fuite mémoire en session longue ; `push` amorti O(1)) |
| `trail.push(pos)` / `trail.as_slice()` / `trail.clear()` | Alimenter et lire le buffer |
| `trail.len()` / `trail.is_empty()` / `trail.capacity()` | Taille courante et capacité |

### Vecteurs (`src/math.rs`)

| Fonction | Rôle |
|---|---|
| `add(a, b)` / `sub(a, b)` | Addition / soustraction de deux `Vec3` |
| `scale(a, s)` | Mise à l'échelle d'un `Vec3` par un scalaire |
| `dot(a, b)` | Produit scalaire |
| `cross(a, b)` | Produit vectoriel |
| `length(a)` | Norme du vecteur |
| `normalize(a)` | Normalise le vecteur (renvoie `a` inchangé si trop proche de zéro) |
| `mat3_mul(a, b)` / `mat3_mul_vec(&m, v)` / `mat3_transpose(m)` | Produit de matrices 3x3, matrice x vecteur, transposée |

### Rotations Euler (`Rotation3D`, `src/math.rs`)

| Fonction | Rôle |
|---|---|
| `Rotation3D::new(roll, pitch, yaw)` | Angles en radians (`const fn`) |
| `Rotation3D::IDENTITY` | Rotation nulle |
| `rotation.rotate_point(p)` | Applique la rotation à un point |
| `rotation.rotate_points(&pts)` / `rotate_points_into(&pts, &mut out)` | Rotation d'un lot de points, matrice calculée une seule fois (la seconde réutilise un buffer) |
| `rotation.compose(other)` | Applique `self` puis `other` (produit de matrices exact). Près du gimbal lock (pitch ≈ ±90°), le résultat reste géométriquement correct mais roll et yaw peuvent être redistribués (yaw figé à 0) |
| `rotation.inverse()` | Rotation inverse |
| `rotation.slerp(other, t)` | Interpolation sphérique par le chemin le plus court (rejeu de logs) |
| `rotation.normalized()` | Même rotation, angles ramenés dans la plage normalisée |
| `rotation.to_matrix()` / `Rotation3D::from_matrix(m)` | Conversion angles ↔ matrice 3x3 |

### Quaternions (`Quat`, `src/quat.rs`)

Format natif des IMU et autopilotes (PX4, ArduPilot, BNO055…), sans gimbal lock.

| Fonction | Rôle |
|---|---|
| `Quat::IDENTITY` / `Quat::new(w, x, y, z)` | Rotation nulle / construction à partir des composantes |
| `Quat::from_axis_angle(axis, angle)` | Rotation autour d'un axe (l'axe est normalisé) |
| `Quat::from_euler(r)` / `q.to_euler()` | Conversion depuis / vers `Rotation3D` (aussi via `From`) |
| `q.compose(other)` | Applique `q` puis `other`, même sémantique que `Rotation3D::compose` |
| `a * b` | Produit de Hamilton : applique `b` d'abord, puis `a` (comme un produit de matrices) |
| `q.inverse()` / `q.conjugate()` | Inverse / conjugué |
| `q.normalize()` | Renormalise (à faire régulièrement si l'on intègre des vitesses angulaires, pour éviter la dérive) |
| `q.slerp(other, t)` | Interpolation sphérique, chemin le plus court |
| `q.rotate_point(p)` / `q.to_matrix()` | Application à un point / matrice 3x3 |

## Exemple minimal

Depuis egui/eframe 0.35, `App::update(ctx, frame)` a été remplacé par
`App::ui(ui, frame)` : on reçoit directement un `&mut egui::Ui` plutôt qu'un
`&egui::Context`, et les panels se dessinent avec `.show(ui, ...)` au lieu de
`.show(ctx, ...)`. Pour accéder au `Context` (ex : `request_repaint`), on
passe par `ui.ctx()`.

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
                r.draw_mesh(&self.mesh, self.state.position, self.state.rotation, egui::Color32::WHITE);
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

## Exemple avec un quaternion (IMU)

```rust
use drew_sim::{Quat, Rotation3D, TelemetryState};

// Quaternion (w, x, y, z) fourni par le capteur, normalisé par précaution.
let q = Quat::new(0.9239, 0.0, 0.3827, 0.0).normalize();
let state = TelemetryState::new([0.0, 1.0, 0.0], q.to_euler());
```

Les composantes `(w, x, y, z)` d'un capteur dépendent de son propre repère
(NED, ENU…) : il faut les exprimer dans le repère de drew-sim (voir
[Conventions](#conventions)) avant de les convertir.

## Exemple complet

Un exemple plus riche (cube animé, panneau de contrôle, trail, graphe
d'altitude) est fourni :

```bash
cargo run --example orbit_viewer --release
```

## Migration 0.3 → 0.4

- `Rotation3D::rotate_point` et `compose` prennent `self` (type `Copy`) au lieu
  de `&self` : les appels `r.rotate_point(p)` sont inchangés, seuls les appels
  écrits `Rotation3D::rotate_point(&r, p)` doivent perdre le `&`.
- `Rotation3D::compose` renvoie des angles **normalisés** (voir
  [Conventions](#conventions)).
- `Mesh3D` a désormais un champ privé (cache d'arêtes) : on ne peut plus
  l'écrire `Mesh3D { vertices, triangles }` hors de la crate. Utiliser
  `Mesh3D::from_raw`, `unit_cube` ou `from_stl_file`.
- `GridStyle` et `MeshError` sont `#[non_exhaustive]` : partir de
  `GridStyle::default()` puis `with_*`, et prévoir un bras `_ =>` dans les
  `match` sur `MeshError`.

## Feuille de route

- [ ] Rendu avec faces pleines / z-buffer simple (actuellement fil de fer)
- [ ] Caméra en perspective avec inclinaison (actuellement orthographique, orbite horizontale)
- [ ] Import d'autres formats de maillage (OBJ)
- [ ] HUD optionnel affichant les contrôles souris et l'état de la caméra
- [ ] Feature optionnelle `glam` (conversions `Quat`/`Vec3`)
- [ ] CI (fmt, clippy, tests, doc) et MSRV déclarée

## Licence

GPL-2.0-or-later, voir [LICENSE](LICENSE).

Copyright (C) 2026 Jorge Andre Castro