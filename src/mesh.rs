use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use crate::utils::{Color, Vec2, Vec3};

/// Primitives are the basic geometric shapes that make up a mesh.
/// Currently only triangles are supported, but lines and points could be added in the future.
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Primitive {
    Point(u32),     // Single vertex (point)
    Line(u32, u32), // Two vertices (line)
    Triangle(u32, u32, u32), // Three vertices (triangle)
}

/// A Mesh is a collection of primitives (triangles, lines, points) that define an object's surface,
/// and their attributes
#[derive(Debug)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
    pub positions: Vec<Vec3>,
    pub colors: Option<Vec<Color>>,
    pub normals: Option<Vec<Vec3>>,
    pub uvs: Option<Vec<Vec2>>,
}
impl Mesh {
    fn new() -> Self {
        Mesh {
            primitives: Vec::new(),
            positions: Vec::new(),
            colors: None,
            normals: None,
            uvs: None,
        }
    }

    pub fn parse_obj(file_path: &str) -> io::Result<Mesh> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut mesh = Mesh::new();
        let mut face_count = 0;

        // Temporary storage for vertices, UVs, and normals
        let mut temp_positions = Vec::new();
        let mut temp_normals = Vec::new();
        let mut temp_uvs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            // Skip empty lines
            if parts.is_empty() { continue; }

            match parts.get(0).map(|s| *s) {
                Some("v") => {
                    // Parse vertex position (v x y z)
                    if parts.len() == 4 {
                        let x: f32 = parts[1].parse().unwrap();
                        let y: f32 = parts[2].parse().unwrap();
                        let z: f32 = parts[3].parse().unwrap();
                        temp_positions.push(Vec3::new(x, y, z));
                    }
                }
                Some("vt") => {
                    // Parse UV (vt u v)
                    if parts.len() == 3 {
                        let u: f32 = parts[1].parse().unwrap();
                        let v: f32 = parts[2].parse().unwrap();
                        temp_uvs.push(Vec2::new(u, v));
                    }
                }
                Some("vn") => {
                    // Parse normal (vn nx ny nz)
                    if parts.len() == 4 {
                        let nx: f32 = parts[1].parse().unwrap();
                        let ny: f32 = parts[2].parse().unwrap();
                        let nz: f32 = parts[3].parse().unwrap();
                        temp_normals.push(Vec3::new(nx, ny, nz));
                    }
                }
                Some("f") => {
                    // Parse face (f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3)
                    if parts.len() >= 4 {
                        let mut indices = Vec::new();
                        for &face_part in &parts[1..] {
                            let face_indices: Vec<&str> = face_part.split('/').collect();
                            let v_idx: usize = face_indices[0].parse::<usize>().unwrap() - 1; // Subtract 1 for 0-based indexing
                            let uv_idx: Option<usize> = if face_indices.len() > 1 && !face_indices[1].is_empty() {
                                Some(face_indices[1].parse::<usize>().unwrap() - 1) // Subtract 1 for 0-based indexing
                            } else {
                                None
                            };
                            let vn_idx: Option<usize> = if face_indices.len() > 2 && !face_indices[2].is_empty() {
                                Some(face_indices[2].parse::<usize>().unwrap() - 1) // Subtract 1 for 0-based indexing
                            } else {
                                None
                            };

                            indices.push((v_idx, uv_idx, vn_idx));
                        }

                        // Handle face (triangle for now)
                        if indices.len() == 3 {
                            let (v1, uv1, vn1) = &indices[0];
                            let (v2, uv2, vn2) = &indices[1];
                            let (v3, uv3, vn3) = &indices[2];

                            // Add triangle primitive
                            mesh.primitives.push(Primitive::Triangle(*v1 as u32, *v2 as u32, *v3 as u32));

                            // Add position indices
                            if mesh.positions.len() <= *v1 {
                                mesh.positions.push(temp_positions[*v1]);
                            }
                            if mesh.positions.len() <= *v2 {
                                mesh.positions.push(temp_positions[*v2]);
                            }
                            if mesh.positions.len() <= *v3 {
                                mesh.positions.push(temp_positions[*v3]);
                            }

                            // Add UVs if present
                            if let Some(uv_idx1) = uv1 {
                                if mesh.uvs.is_none() {
                                    mesh.uvs = Some(temp_uvs.clone());
                                }
                                if mesh.uvs.as_ref().unwrap().len() <= *uv_idx1 {
                                    mesh.uvs.as_mut().unwrap().push(temp_uvs[*uv_idx1]);
                                }
                            }

                            // Add normals if present
                            if let Some(vn_idx1) = vn1 {
                                if mesh.normals.is_none() {
                                    mesh.normals = Some(temp_normals.clone());
                                }
                                if mesh.normals.as_ref().unwrap().len() <= *vn_idx1 {
                                    mesh.normals.as_mut().unwrap().push(temp_normals[*vn_idx1]);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        mesh.positions = temp_positions;
        if !temp_uvs.is_empty() {
            mesh.uvs = Some(temp_uvs);
        }
        if !temp_normals.is_empty() {
            mesh.normals = Some(temp_normals);
        }
        Ok(mesh)
    }
}

