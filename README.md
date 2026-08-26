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
drew-sim = "0.3"
```

La feature `stl` (chargement de fichiers `.stl`) est activée par défaut.
Pour s'en passer :

```toml
drew-sim = { version = "0.3", default-features = false }
```

## Contrôles souris (zone de rendu 3D)

| Action                                    | Effet             |
|--------------------------------------------|-------------------|
| Clic gauche + glisser                      | Orbite la caméra  |
| Molette                                    | Zoom              |
| Clic droit **ou** clic molette + glisser   | Panoramique       |

Ces contrôles sont gérés automatiquement par `Engine3D::render`,rien à
faire côté application, mais rien n'est affiché à l'écran non plus (pas de
widget de caméra visible : c'est purement à la souris).

## Fonctions utiles

### `Engine3D` : moteur haut niveau (`src/engine.rs`)

| Fonction | Rôle |
|---|---|
| `Engine3D::default()` | Caméra à `orbit=0.5`, `zoom=1.0`, grille par défaut |
| `engine.render(ui, \|r\| { ... })` | Alloue la zone de dessin, applique la souris à la caméra, dessine la grille de sol, puis appelle la closure avec un `Render3D` prêt à l'emploi |
| `engine.camera` | Accès direct à l'`OrbitCamera` (ex : forcer un angle au démarrage) |
| `engine.grid` | `GridStyle` : couleurs et espacement de la grille au sol |

### `Render3D` : primitives de dessin (`src/render.rs`), utilisées dans la closure de `render(...)`

| Fonction | Rôle |
|---|---|
| `r.world_to_screen(pos)` | Projette un point 3D vers l'écran (utile pour dessiner autre chose que les primitives ci-dessous) |
| `r.draw_mesh(&mesh, position, rotation, color)` | Maillage en fil de fer |
| `r.draw_mesh_with_stroke(&mesh, position, rotation, stroke)` | Idem, avec un `egui::Stroke` complet (épaisseur de trait) |
| `r.draw_axes(position, rotation, length)` | Repère XYZ (Rouge = X, Vert = Y, Bleu = Z) |
| `r.draw_trail(&points, color)` | Polyligne reliant un historique de positions |
| `r.draw_chart(rect, &values, min, max, color)` | Graphe 2D (courbe de valeurs), indépendant de la caméra 3D , pour afficher un capteur en overlay |

### `Mesh3D` (`src/mesh.rs`)

| Fonction | Rôle |
|---|---|
| `Mesh3D::from_stl_file(path)` | Charge un `.stl` (nécessite la feature `stl`) |
| `Mesh3D::unit_cube(size)` | Cube généré sans dépendance externe, pratique pour tester rapidement |
| `Mesh3D::from_raw(vertices, triangles)` | Construit un maillage à la main, avec validation des indices |
| `mesh.triangle_count()` | Nombre de triangles |

### `TelemetryState` / `TrailBuffer` (`src/telemetry.rs`)

| Fonction | Rôle |
|---|---|
| `TelemetryState { position, rotation }` | Contrat agnostique de la source (capteur réel, simulation, rejeu de log) |
| `TrailBuffer::new(capacity)` | Historique de positions à taille bornée (évite une fuite mémoire en session longue) |
| `trail.push(pos)` / `trail.as_slice()` / `trail.clear()` | Alimenter et lire le buffer |

### Vecteurs et rotations (`src/math.rs`)

| Fonction | Rôle |
|---|---|
| `add(a, b)` / `sub(a, b)` | Addition / soustraction de deux `Vec3` |
| `scale(a, s)` | Mise à l'échelle d'un `Vec3` par un scalaire |
| `dot(a, b)` | Produit scalaire |
| `cross(a, b)` | Produit vectoriel |
| `length(a)` | Norme du vecteur |
| `normalize(a)` | Normalise le vecteur (renvoie `a` inchangé si trop proche de zéro) |
| `Rotation3D::new(roll, pitch, yaw)` | Angles en radians |
| `Rotation3D::IDENTITY` | Rotation nulle |
| `rotation.rotate_point(p)` | Applique la rotation à un point |
| `rotation.compose(other)` | Cumule deux rotations (composition matricielle exacte, gère le gimbal lock) |

## Exemple minimal

Depuis egui/eframe 0.35, `App::update(ctx, frame)` a été remplacé par
`App::ui(ui, frame)` : on reçoit directement un `&mut egui::Ui` plutôt qu'un
`&egui::Context`, et les panels se dessinent avec `.show(ui, ...)` au lieu de
`.show(ctx, ...)`. Pour accéder au `Context` (ex : `request_repaint`), on
passe par `ui.ctx()`.

```rust
use drew_sim::{Engine3D, Mesh3D, Rotation3D, TelemetryState};
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

## Exemple complet

Un exemple plus riche (cube animé, panneau de contrôle, trail, graphe
d'altitude) est fourni :

```bash
cargo run --example orbit_viewer --release
```

## Feuille de route

- [ ] Rendu avec faces pleines / z-buffer simple (actuellement fil de fer)
- [ ] Import d'autres formats de maillage (OBJ)
- [ ] HUD optionnel affichant les contrôles souris et l'état de la caméra

## Licence

GPL-2.0-or-later voir [LICENSE](LICENSE).

Copyright (C) 2026 Jorge Andre Castro