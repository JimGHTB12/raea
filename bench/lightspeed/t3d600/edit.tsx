// T3D600: a lit cube turning under moving text, 1080x1920 at 30 fps for 600 frames, lightspeed T2's 3D export
// fixture (bench/lightspeed/export.sh); the 60-frame t3d it grew from is the same scene

import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";

import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Quat, Transform } from "raea/transform";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Camera3d } from "raea/camera";
function Edit() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#202040" }}>
      <div style={{ position: "absolute", left: 0, top: 0, width: 1080, height: 1920 }}>
        <>
          <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
          <AmbientLight brightness={160} />
          <DirectionalLight transform={Transform.fromXyz(2, 3, 4).lookingAt([0, 0, 0])} illuminance={24000} shadow_maps_enabled />
          <Mesh3d transform={Transform.fromXyz(0, 0, 0).withRotation(Quat.fromEuler("XYZ", f * 0.05, f * 0.08, 0))}>
            <Cuboid half_size={[1, 1, 1]} />
            <StandardMaterial base_color="#e05030" />
          </Mesh3d>
        </>
      </div>
      <div style={{ position: "absolute", left: 100, top: 300 + (f % 300) * 4, fontSize: 120, color: "white", fontFamily: "Helvetica" }}>Hello {f}</div>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="t3d600" component={Edit} width={1080} height={1920} fps={30} durationInFrames={600} />);
