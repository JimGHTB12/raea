// Bevy v0.20.0 examples/3d/lightmaps.rs in raea's TSX (docs/control.feature FC0a), its default arguments (forward,
// no bicubic): models/CornellBox/CornellBox.glb's scene with no ambient light, and the camera at (-278, 273, 800).
// add_lightmaps_to_meshes gives each mesh by its GltfMeshName a Lightmap (large_box, small_box and cornell_box, each
// its lightmaps/CornellBox-*.zstd.ktx2) and sets its material's lightmap_exposure to 250: the box material those two
// boxes share and the cornell box's three (the Light mesh's own emitter is left as it is). In Bevy's default
// 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Lightmap, StandardMaterial } from "raea/pbr";
import { GlobalAmbientLight } from "raea/light";

const BAKED = ["box.Material", "cbox.Material.002", "cbox.red.002", "cbox.green.002"];

function Shot() {
  return (
    <>
      <GlobalAmbientLight brightness={0} />
      <Model src={staticFile("models/CornellBox/CornellBox.glb")}>
        <Lightmap mesh="large_box" image={staticFile("lightmaps/CornellBox-Large.zstd.ktx2")} />
        <Lightmap mesh="small_box" image={staticFile("lightmaps/CornellBox-Small.zstd.ktx2")} />
        <Lightmap mesh="cornell_box" image={staticFile("lightmaps/CornellBox-Box.zstd.ktx2")} />
        {BAKED.map((m) => <StandardMaterial key={m} material={m} lightmap_exposure={250} />)}
      </Model>
      <Camera3d transform={Transform.fromXyz(-278, 273, 800)} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
