//! Triangular meshes: storage, primitives, and STL loading.

use crate::math::Vec3;
use std::sync::OnceLock;

#[cfg(feature = "stl")]
use std::path::Path;

/// Errors that may occur while loading a mesh.
#[derive(Debug)]
#[non_exhaustive]
pub enum MeshError {
    /// I/O error (file not found, unreadable, etc.).
    Io(std::io::Error),
    /// Invalid content (corrupt file, out-of-bounds index, etc.).
    Parse(String),
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::Io(e) => write!(f, "I/O error: {e}"),
            MeshError::Parse(e) => write!(f, "mesh read error: {e}"),
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

/// 3D mesh: a list of vertices and triangles (indices into `vertices`).
///
/// Unique edges (used for wireframe rendering) are computed once on the first
/// request and then cached. **If you modify `triangles` afterward, call
/// [`Mesh3D::invalidate_edges`].**
#[derive(Clone, Debug, Default)]
pub struct Mesh3D {
    /// Mesh vertices.
    pub vertices: Vec<Vec3>,
    /// Triangles, each represented by three indices into `vertices`.
    pub triangles: Vec<[usize; 3]>,
    edges: OnceLock<Vec<[usize; 2]>>,
}

impl Mesh3D {
    /// Creates an empty mesh.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a mesh from raw vertices and triangles.
    ///
    /// # Errors
    /// Returns an error if a triangle references an out-of-bounds index.
    pub fn from_raw(vertices: Vec<Vec3>, triangles: Vec<[usize; 3]>) -> Result<Self, MeshError> {
        let n = vertices.len();
        for tri in &triangles {
            if tri.iter().any(|&i| i >= n) {
                return Err(MeshError::Parse(format!(
                    "triangle index out of bounds: {tri:?} (n_vertices={n})"
                )));
            }
        }
        Ok(Self { vertices, triangles, edges: OnceLock::new() })
    }

    /// Loads a model from a `.stl` file (binary or ASCII).
    ///
    /// # Errors
    /// Returns an I/O or file-reading error.
    #[cfg(feature = "stl")]
    pub fn from_stl_file<P: AsRef<Path>>(path: P) -> Result<Self, MeshError> {
        let mut file = std::fs::File::open(path)?;
        let stl = stl_io::read_stl(&mut file).map_err(|e| MeshError::Parse(e.to_string()))?;

        let vertices = stl.vertices.iter().map(|v| [v[0], v[1], v[2]]).collect();
        let triangles = stl.faces.iter().map(|f| f.vertices).collect();

        Self::from_raw(vertices, triangles)
    }

    /// A cube centered at the origin with edge length `size`, useful for tests
    /// and examples that do not depend on an STL file.
    #[must_use]
    pub fn unit_cube(size: f32) -> Self {
        let h = size * 0.5;
        let vertices = vec![
            [-h, -h, -h], [h, -h, -h], [h, h, -h], [-h, h, -h],
            [-h, -h, h],  [h, -h, h],  [h, h, h],  [-h, h, h],
        ];
        let triangles = vec![
            [0, 1, 2], [0, 2, 3], // back
            [4, 6, 5], [4, 7, 6], // front
            [0, 4, 5], [0, 5, 1], // bottom
            [3, 2, 6], [3, 6, 7], // top
            [0, 3, 7], [0, 7, 4], // left
            [1, 5, 6], [1, 6, 2], // right
        ];
        Self { vertices, triangles, edges: OnceLock::new() }
    }

    /// Number of triangles.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Unique mesh edges (each edge shared by two triangles appears only once),
    /// sorted with `a < b`. Computed on first request and then cached.
    #[must_use]
    pub fn edges(&self) -> &[[usize; 2]] {
        self.edges.get_or_init(|| unique_edges(&self.triangles))
    }

    /// Clears the edge cache. Call after modifying `triangles`.
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
        // 12 cube edges + 6 face diagonals (12 triangles = 36 raw edges).
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