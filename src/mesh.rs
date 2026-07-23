//! Maillages triangulaires : stockage, primitives, chargement STL.

use crate::math::Vec3;
use std::path::Path;

/// Erreurs pouvant survenir lors du chargement d'un maillage.
#[derive(Debug)]
pub enum MeshError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::Io(e) => write!(f, "erreur d'entrée/sortie: {e}"),
            MeshError::Parse(e) => write!(f, "erreur de lecture du maillage: {e}"),
        }
    }
}

impl std::error::Error for MeshError {}

impl From<std::io::Error> for MeshError {
    fn from(e: std::io::Error) -> Self {
        MeshError::Io(e)
    }
}

/// Maillage 3D : liste de sommets et de triangles (indices dans `vertices`).
#[derive(Clone, Debug, Default)]
pub struct Mesh3D {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
}

impl Mesh3D {
    /// Construit un maillage vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construit un maillage à partir de sommets et de triangles bruts.
    ///
    /// Retourne une erreur si un triangle référence un index hors bornes.
    pub fn from_raw(vertices: Vec<Vec3>, triangles: Vec<[usize; 3]>) -> Result<Self, MeshError> {
        let n = vertices.len();
        for tri in &triangles {
            if tri.iter().any(|&i| i >= n) {
                return Err(MeshError::Parse(format!(
                    "index de triangle hors bornes: {tri:?} (n_vertices={n})"
                )));
            }
        }
        Ok(Self { vertices, triangles })
    }

    /// Charge dynamiquement un modèle depuis un fichier `.stl` (binaire ou ASCII).
    #[cfg(feature = "stl")]
    pub fn from_stl_file<P: AsRef<Path>>(path: P) -> Result<Self, MeshError> {
        let mut file = std::fs::File::open(path)?;
        let stl = stl_io::read_stl(&mut file).map_err(|e| MeshError::Parse(e.to_string()))?;

        let vertices = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
        let triangles = stl.faces.iter().map(|f| f.vertices).collect();

        Ok(Self { vertices, triangles })
    }

    /// Un cube unité centré à l'origine (arête de longueur `size`), utile pour
    /// les tests et les exemples sans dépendre d'un fichier STL.
    pub fn unit_cube(size: f32) -> Self {
        let h = size * 0.5;
        let vertices = vec![
            [-h, -h, -h], [h, -h, -h], [h, h, -h], [-h, h, -h],
            [-h, -h, h],  [h, -h, h],  [h, h, h],  [-h, h, h],
        ];
        let triangles = vec![
            [0, 1, 2], [0, 2, 3], // arrière
            [4, 6, 5], [4, 7, 6], // avant
            [0, 4, 5], [0, 5, 1], // dessous
            [3, 2, 6], [3, 6, 7], // dessus
            [0, 3, 7], [0, 7, 4], // gauche
            [1, 5, 6], [1, 6, 2], // droite
        ];
        Self { vertices, triangles }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }
}