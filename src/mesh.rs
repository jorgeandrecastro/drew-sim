//! Maillages triangulaires : stockage, primitives, chargement STL.

use crate::math::Vec3;
use std::sync::OnceLock;

#[cfg(feature = "stl")]
use std::path::Path;

/// Erreurs pouvant survenir lors du chargement d'un maillage.
#[derive(Debug)]
#[non_exhaustive]
pub enum MeshError {
    /// Erreur d'entrée/sortie (fichier introuvable, illisible…).
    Io(std::io::Error),
    /// Contenu invalide (fichier corrompu, index hors bornes…).
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

impl std::error::Error for MeshError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MeshError::Io(e) => Some(e),
            MeshError::Parse(_) => None,
        }
    }
}

impl From<std::io::Error> for MeshError {
    fn from(e: std::io::Error) -> Self {
        MeshError::Io(e)
    }
}

/// Maillage 3D : liste de sommets et de triangles (indices dans `vertices`).
///
/// Les arêtes uniques (utilisées par le rendu fil de fer) sont calculées
/// une fois, à la première demande, puis mises en cache. **Si tu modifies
/// `triangles` après coup, appelle [`Mesh3D::invalidate_edges`].**
#[derive(Clone, Debug, Default)]
pub struct Mesh3D {
    /// Sommets du maillage.
    pub vertices: Vec<Vec3>,
    /// Triangles, sous forme de trois indices dans `vertices`.
    pub triangles: Vec<[usize; 3]>,
    edges: OnceLock<Vec<[usize; 2]>>,
}

impl Mesh3D {
    /// Construit un maillage vide.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construit un maillage à partir de sommets et de triangles bruts.
    ///
    /// # Errors
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
        Ok(Self { vertices, triangles, edges: OnceLock::new() })
    }

    /// Charge un modèle depuis un fichier `.stl` (binaire ou ASCII).
    ///
    /// # Errors
    /// Erreur d'E/S ou de lecture du fichier.
    #[cfg(feature = "stl")]
    pub fn from_stl_file<P: AsRef<Path>>(path: P) -> Result<Self, MeshError> {
        let mut file = std::fs::File::open(path)?;
        let stl = stl_io::read_stl(&mut file).map_err(|e| MeshError::Parse(e.to_string()))?;

        let vertices = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
        let triangles = stl.faces.iter().map(|f| f.vertices).collect();

        Self::from_raw(vertices, triangles)
    }

    /// Un cube centré à l'origine (arête de longueur `size`), utile pour
    /// les tests et les exemples sans dépendre d'un fichier STL.
    #[must_use]
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
        Self { vertices, triangles, edges: OnceLock::new() }
    }

    /// Nombre de triangles.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Arêtes uniques du maillage (chaque arête partagée par deux triangles
    /// n'apparaît qu'une fois), triées, avec `a < b`. Calculées à la
    /// première demande puis mises en cache.
    #[must_use]
    pub fn edges(&self) -> &[[usize; 2]] {
        self.edges.get_or_init(|| unique_edges(&self.triangles))
    }

    /// Oublie le cache d'arêtes. À appeler après avoir modifié `triangles`.
    pub fn invalidate_edges(&mut self) {
        self.edges = OnceLock::new();
    }
}

fn unique_edges(triangles: &[[usize; 3]]) -> Vec<[usize; 2]> {
    let mut edges = Vec::with_capacity(triangles.len() * 3);
    for &[a, b, c] in triangles {
        for (u, v) in [(a, b), (b, c), (c, a)] {
            if u != v {
                edges.push(if u < v { [u, v] } else { [v, u] });
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_18_unique_edges() {
        // 12 arêtes du cube + 6 diagonales de faces (12 triangles = 36 arêtes brutes).
        let cube = Mesh3D::unit_cube(1.0);
        assert_eq!(cube.triangle_count(), 12);
        assert_eq!(cube.edges().len(), 18);
    }

    #[test]
    fn shared_edge_counted_once() {
        let m = Mesh3D::from_raw(
            vec![[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2], [0, 2, 3]],
        )
        .unwrap();
        assert_eq!(m.edges(), &[[0, 1], [0, 2], [0, 3], [1, 2], [2, 3]]);
    }

    #[test]
    fn degenerate_edges_are_skipped() {
        assert_eq!(unique_edges(&[[0, 0, 1]]), vec![[0, 1]]);
    }

    #[test]
    fn invalidate_edges_recomputes() {
        let mut m = Mesh3D::from_raw(vec![[0.0; 3]; 4], vec![[0, 1, 2]]).unwrap();
        assert_eq!(m.edges().len(), 3);
        m.triangles.push([1, 2, 3]);
        m.invalidate_edges();
        assert_eq!(m.edges().len(), 5);
    }

    #[test]
    fn from_raw_rejects_out_of_range_index() {
        assert!(Mesh3D::from_raw(vec![[0.0; 3]; 2], vec![[0, 1, 2]]).is_err());
    }
}