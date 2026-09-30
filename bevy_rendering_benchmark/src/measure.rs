//! What is being rendered: meshes, vertices, triangles, materials and textures of everything
//! visible. Copied from bevy_diplomovka's debug_tools.rs `measure_model`, but computed on demand
//! instead of through diagnostics.

use bevy::ecs::entity::Entities;
use bevy::ecs::system::SystemParam;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use std::collections::HashSet;

pub struct ModelStats {
    pub mesh_instances: usize,
    pub unique_meshes: usize,
    pub vertices: usize,
    pub triangles: usize,
    pub materials: usize,
    pub textures: usize,
    pub texture_mib: f64,
    pub entities: u32,
}

#[derive(SystemParam)]
pub struct MeasureModel<'w, 's> {
    q_meshes: Query<
        'w,
        's,
        (
            &'static Mesh3d,
            Option<&'static MeshMaterial3d<StandardMaterial>>,
            &'static InheritedVisibility,
        ),
    >,
    meshes: Res<'w, Assets<Mesh>>,
    materials: Res<'w, Assets<StandardMaterial>>,
    images: Res<'w, Assets<Image>>,
    entities: &'w Entities,
}

impl MeasureModel<'_, '_> {
    pub fn measure(&self) -> ModelStats {
        let mut instances = 0usize;
        let mut vertices = 0usize;
        let mut triangles = 0usize;
        let mut unique_meshes = HashSet::new();
        let mut used_materials = HashSet::new();

        for (mesh_handle, material, visibility) in &self.q_meshes {
            if !visibility.get() {
                continue;
            }
            instances += 1;
            unique_meshes.insert(mesh_handle.id());
            if let Some(material) = material {
                used_materials.insert(material.id());
            }
            if let Some(mesh) = self.meshes.get(mesh_handle) {
                let (v, t) = mesh_counts(mesh);
                vertices += v;
                triangles += t;
            }
        }

        let mut textures = HashSet::new();
        for material in used_materials.iter().filter_map(|id| self.materials.get(*id)) {
            for texture in [
                &material.base_color_texture,
                &material.emissive_texture,
                &material.metallic_roughness_texture,
                &material.normal_map_texture,
                &material.occlusion_texture,
            ]
            .into_iter()
            .flatten()
            {
                textures.insert(texture.id());
            }
        }
        let texture_bytes: u64 = textures
            .iter()
            .filter_map(|id| self.images.get(*id))
            .map(texture_size)
            .sum();

        ModelStats {
            mesh_instances: instances,
            unique_meshes: unique_meshes.len(),
            vertices,
            triangles,
            materials: used_materials.len(),
            textures: textures.len(),
            texture_mib: texture_bytes as f64 / (1024. * 1024.),
            entities: self.entities.count_spawned(),
        }
    }
}

/// Vertex and triangle count, zero if the mesh data only lives in the render world.
pub fn mesh_counts(mesh: &Mesh) -> (usize, usize) {
    let Ok(positions) = mesh.try_attribute(Mesh::ATTRIBUTE_POSITION) else {
        return (0, 0);
    };
    let vertices = positions.len();
    let indices = match mesh.try_indices_option() {
        Ok(Some(indices)) => indices.len(),
        _ => vertices,
    };
    let triangles = match mesh.primitive_topology() {
        PrimitiveTopology::TriangleList => indices / 3,
        PrimitiveTopology::TriangleStrip => indices.saturating_sub(2),
        _ => 0,
    };
    (vertices, triangles)
}

/// GPU size estimate from the texture descriptor, the pixel data itself may already be dropped
/// from the main world after upload.
fn texture_size(image: &Image) -> u64 {
    let descriptor = &image.texture_descriptor;
    let format = descriptor.format;
    let (block_w, block_h) = format.block_dimensions();
    let block_bytes = format.block_copy_size(None).unwrap_or(4) as u64;
    let mut width = descriptor.size.width;
    let mut height = descriptor.size.height;
    let layers = descriptor.size.depth_or_array_layers as u64;
    let mut total = 0;
    for _ in 0..descriptor.mip_level_count.max(1) {
        let blocks_x = width.div_ceil(block_w) as u64;
        let blocks_y = height.div_ceil(block_h) as u64;
        total += blocks_x * blocks_y * block_bytes * layers;
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }
    total
}
