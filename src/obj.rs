use crate::vertex::Vertex;
use raylib::math::{Vector2, Vector3};
use tobj;

pub struct Obj {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Obj {
    #[allow(dead_code)]
    pub fn load(path: &str) -> Result<Self, tobj::LoadError> {
        let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            for i in 0..num_vertices {
                let x = mesh.positions[i * 3];
                let y = mesh.positions[i * 3 + 1];
                let z = mesh.positions[i * 3 + 2];
                let position = Vector3::new(x, -y, z);

                let normal = if !mesh.normals.is_empty() {
                    let nx = mesh.normals[i * 3];
                    let ny = mesh.normals[i * 3 + 1];
                    let nz = mesh.normals[i * 3 + 2];
                    Vector3::new(nx, ny, nz)
                } else {
                    Vector3::zero()
                };

                let tex_coords = if !mesh.texcoords.is_empty() {
                    let u = mesh.texcoords[i * 2];
                    let v = mesh.texcoords[i * 2 + 1];
                    Vector2::new(u, v)
                } else {
                    Vector2::zero()
                };

                vertices.push(Vertex::new(position, normal, tex_coords));
            }
            indices.extend_from_slice(&mesh.indices);
        }

        Ok(Obj { vertices, indices })
    }

    pub fn get_vertex_array(&self) -> Vec<Vertex> {
        let mut vertex_array = Vec::new();
        for &index in &self.indices {
            vertex_array.push(self.vertices[index as usize].clone());
        }
        vertex_array
    }

    /// Generates a sphere mesh programmatically
    /// radius: radius of the sphere
    /// segments: number of segments in both latitude and longitude (higher = smoother sphere)
    pub fn generate_sphere(radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices
        for i in 0..=segments {
            let theta = std::f32::consts::PI * i as f32 / segments as f32; // Vertical angle (0 to PI)
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for j in 0..=segments {
                let phi = 2.0 * std::f32::consts::PI * j as f32 / segments as f32; // Horizontal angle (0 to 2*PI)
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                // Position
                let x = radius * sin_theta * cos_phi;
                let y = radius * cos_theta;
                let z = radius * sin_theta * sin_phi;
                let position = Vector3::new(x, y, z);

                // Normal (same as position normalized, since sphere is centered at origin)
                let normal = Vector3::new(x / radius, y / radius, z / radius);

                // Texture coordinates (for procedural shaders, we can use spherical coordinates)
                let u = j as f32 / segments as f32;
                let v = i as f32 / segments as f32;
                let tex_coords = Vector2::new(u, v);

                vertices.push(Vertex::new(position, normal, tex_coords));
            }
        }

        // Generate indices for triangles
        for i in 0..segments {
            for j in 0..segments {
                let first = (i * (segments + 1) + j) as u32;
                let second = (first + 1) as u32;
                let third = ((i + 1) * (segments + 1) + j) as u32;
                let fourth = (third + 1) as u32;

                // First triangle
                indices.push(first);
                indices.push(second);
                indices.push(third);

                // Second triangle
                indices.push(second);
                indices.push(fourth);
                indices.push(third);
            }
        }

        Obj { vertices, indices }
    }

    /// Genera anillos planetarios usando un disco fino
    /// inner_radius: radio interno del anillo
    /// outer_radius: radio externo del anillo
    /// segments_radial: número de segmentos radiales
    /// segments_angular: número de segmentos angulares
    pub fn generate_rings(inner_radius: f32, outer_radius: f32, segments_radial: u32, segments_angular: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generar vértices en un disco delgado (plano Y=0)
        for i in 0..=segments_radial {
            let radius_t = i as f32 / segments_radial as f32;
            let radius = inner_radius + (outer_radius - inner_radius) * radius_t;
            
            for j in 0..=segments_angular {
                let angle = 2.0 * std::f32::consts::PI * j as f32 / segments_angular as f32;
                let x = radius * angle.cos();
                let y = 0.0; // Anillo plano
                let z = radius * angle.sin();
                let position = Vector3::new(x, y, z);
                
                // Normal apuntando hacia arriba
                let normal = Vector3::new(0.0, 1.0, 0.0);
                
                // Texture coordinates
                let u = radius_t;
                let v = j as f32 / segments_angular as f32;
                let tex_coords = Vector2::new(u, v);
                
                vertices.push(Vertex::new(position, normal, tex_coords));
            }
        }

        // Generar índices
        for i in 0..segments_radial {
            for j in 0..segments_angular {
                let first = (i * (segments_angular + 1) + j) as u32;
                let second = (first + 1) as u32;
                let third = ((i + 1) * (segments_angular + 1) + j) as u32;
                let fourth = (third + 1) as u32;

                indices.push(first);
                indices.push(second);
                indices.push(third);

                indices.push(second);
                indices.push(fourth);
                indices.push(third);
            }
        }

        Obj { vertices, indices }
    }
}
