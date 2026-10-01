// What is being rendered: meshes, vertices, triangles, materials and textures of everything
// visible, the same numbers as bevy_rendering_benchmark's measure.rs.

import { InstancedMesh } from "@babylonjs/core";

export function measureModel(scene) {
  let meshInstances = 0;
  let vertices = 0;
  let triangles = 0;
  const geometries = new Set();
  const materials = new Set();

  for (const mesh of scene.meshes) {
    if (!mesh.isEnabled() || !mesh.isVisible) continue;
    const [v, t] = meshCounts(mesh);
    // glTF nodes without a mesh are empty Meshes in Babylon, Bevy has no Mesh3d on them
    if (v === 0) continue;
    meshInstances += 1;
    vertices += v;
    triangles += t;
    const source = mesh instanceof InstancedMesh ? mesh.sourceMesh : mesh;
    if (source.geometry) geometries.add(source.geometry);
    if (mesh.material) materials.add(mesh.material);
  }

  const textures = new Set();
  for (const material of materials) {
    for (const texture of material.getActiveTextures()) {
      textures.add(texture);
    }
  }
  let textureBytes = 0;
  for (const texture of textures) {
    textureBytes += textureSize(texture);
  }

  return {
    meshInstances,
    uniqueMeshes: geometries.size,
    vertices,
    triangles,
    materials: materials.size,
    textures: textures.size,
    textureMib: textureBytes / (1024 * 1024),
    // Bevy counts entities, the closest thing here are scene nodes
    entities:
      scene.meshes.length +
      scene.transformNodes.length +
      scene.lights.length +
      scene.cameras.length,
  };
}

/** `[vertices, triangles]` of one mesh, instances count the vertices of their source mesh */
export function meshCounts(mesh) {
  const vertices = mesh.getTotalVertices();
  if (vertices === 0) return [0, 0];
  const indices = mesh.getTotalIndices() || vertices;
  return [vertices, Math.floor(indices / 3)];
}

/** GPU size estimate, RGBA8 with a full mip chain unless the texture has none */
function textureSize(texture) {
  const { width, height } = texture.getSize();
  const bytes = width * height * 4;
  return texture.noMipmap ? bytes : Math.round((bytes * 4) / 3);
}
