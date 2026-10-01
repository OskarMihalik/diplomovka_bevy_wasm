// The same scene as bevy_rendering_benchmark (bevy_diplomovka's building.rs `setup_scene`): one
// point light, the model at the origin scaled to 0.25 and a camera orbiting the origin. Babylon's
// defaults are replaced by Bevy's where they decide what is drawn and how much it costs: right
// handed coordinates like glTF and Bevy, the default PointLight, shadow map and camera, MSAA in
// an offscreen texture that is copied to the canvas.

import {
  Color3,
  Color4,
  FreeCamera,
  HemisphericLight,
  ImageProcessingConfiguration,
  PassPostProcess,
  PointLight,
  ShadowGenerator,
  TransformNode,
  Vector3,
} from "@babylonjs/core";

// Where bevy_diplomovka spawns its PanOrbitCamera, the orbit pitch comes from it
const CAMERA_START = new Vector3(0, 1.5, 5);
/** Orbit radius of bevy_diplomovka, `CAMERA_START.length()` */
export const DEFAULT_DISTANCE = 5.220153;
const MODEL_SCALE = 0.25;

// Bevy's PointLight::default(): 1 000 000 lm, range 20 m, 2048 px shadow cube map. Bevy's default
// camera exposure (ev100 9.7) scales light by 1 / (1.2 * 2^9.7), Babylon's PBR takes the result
// in candela: 1 000 000 / 4π * exposure ≈ 80.
const EXPOSURE = 1 / (1.2 * 2 ** 9.7);
const POINT_LIGHT_INTENSITY = (1_000_000 / (4 * Math.PI)) * EXPOSURE;
const POINT_LIGHT_RANGE = 20;
const SHADOW_MAP_SIZE = 2048;
// Bevy's default AmbientLight, 80 cd/m², through the same exposure
const AMBIENT_INTENSITY = 80 * EXPOSURE;
// Bevy's default ClearColor
const CLEAR_COLOR = Color4.FromInts(43, 44, 47, 255);

/** Light, camera and MSAA; returns what the benchmark moves or fills later */
export function setupScene(scene, config) {
  scene.useRightHandedSystem = true;
  scene.clearColor = CLEAR_COLOR;
  // no picking in bevy_rendering_benchmark either
  scene.skipPointerMovePicking = true;
  scene.skipPointerDownPicking = true;
  scene.skipPointerUpPicking = true;
  // Bevy's camera tonemaps by default (TonyMcMapface), Babylon has no such curve, ACES is closest
  scene.imageProcessingConfiguration.toneMappingEnabled = true;
  scene.imageProcessingConfiguration.toneMappingType = ImageProcessingConfiguration.TONEMAPPING_ACES;

  const light = new PointLight("light", new Vector3(4, 8, 4), scene);
  light.intensity = POINT_LIGHT_INTENSITY;
  light.range = POINT_LIGHT_RANGE;
  light.shadowMinZ = 0.1;
  light.shadowMaxZ = POINT_LIGHT_RANGE;
  let shadowGenerator = null;
  if (config.shadows) {
    shadowGenerator = new ShadowGenerator(SHADOW_MAP_SIZE, light);
    // Bevy filters point light shadows too, PCF isn't available for point lights in Babylon
    shadowGenerator.usePoissonSampling = true;
  }

  // a uniform ambient term, like Bevy's AmbientLight
  const ambient = new HemisphericLight("ambient", Vector3.Up(), scene);
  ambient.intensity = AMBIENT_INTENSITY;
  ambient.groundColor = Color3.White();
  ambient.specular = Color3.Black();

  // Bevy's Projection::default(): 45° vertical fov, near 0.1. Bevy's projection has no far plane
  // (infinite reverse-Z, `far` only culls whole meshes), a far plane here would cut the model
  // apart at big distances
  scene.getEngine().useReverseDepthBuffer = true;
  const camera = new FreeCamera("camera", Vector3.Zero(), scene);
  camera.fov = Math.PI / 4;
  camera.minZ = 0.1;
  camera.maxZ = 0;
  placeCamera(camera, 0, config.distance);

  // Bevy renders into an offscreen (MSAA) texture and copies it to the canvas, the canvas itself
  // has no antialiasing (see main.js)
  const pass = new PassPostProcess("msaa", 1.0, camera);
  pass.samples = config.msaa;

  return { camera, shadowGenerator };
}

/** Puts the loaded model into the scene at the origin, scaled like in bevy_diplomovka */
export function spawnModel(scene, container, shadowGenerator) {
  const model = new TransformNode("model", scene);
  model.scaling.setAll(MODEL_SCALE);
  const roots = [...container.rootNodes];
  container.addAllToScene();
  for (const node of roots) {
    node.parent = model;
  }
  for (const mesh of container.meshes) {
    if (shadowGenerator) {
      shadowGenerator.addShadowCaster(mesh, false);
      mesh.receiveShadows = true;
    }
  }
  return model;
}

/** Camera at `yaw` radians around the Y axis and `radius` meters from the origin, same math as
 * PanOrbitCamera */
export function placeCamera(camera, yaw, radius) {
  const pitch = Math.asin(CAMERA_START.y / CAMERA_START.length());
  const angle = Math.atan2(CAMERA_START.x, CAMERA_START.z) + yaw;
  camera.position.set(
    radius * Math.cos(pitch) * Math.sin(angle),
    radius * Math.sin(pitch),
    radius * Math.cos(pitch) * Math.cos(angle),
  );
  camera.setTarget(Vector3.Zero());
}
