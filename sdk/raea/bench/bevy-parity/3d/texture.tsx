// Bevy v0.20.0 examples/3d/texture.rs in raea's TSX (docs/control.feature FC0a): three 8 x 2 m quads of Bevy's logo
// (branding/bevy_logo_dark_big.png), unlit and blended, tilted -PI/5 about X at z 1.5, 0 and -1.5, the middle one
// tinted red and the back one blue at 0.5 alpha, the camera at (3, 5, 8) looking at the origin, in Bevy's default
// 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Quat } from "raea/transform";
import { Srgba } from "raea/color";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const LOGO = staticFile("branding/bevy_logo_dark_big.png");
// a Rectangle (XY, facing +Z) is a Plane3d (XZ, facing +Y) turned +PI/2 about X
const UPRIGHT = Quat.fromRotationX(Math.PI / 2);
const TILT = Quat.fromRotationX(-Math.PI / 5);

function Quad({ z, tint }: { z: number; tint?: string }) {
  return (
    <Mesh3d transform={{ translation: [0, 0, z], rotation: Quat.mul(TILT, UPRIGHT) }}>
      <Plane3d half_size={[4, 1]} />
      <StandardMaterial base_color={tint ? Srgba.hex(tint).with_alpha(0.5) : "#ffffff"} base_color_texture={LOGO} alpha_mode="Blend" unlit />
    </Mesh3d>
  );
}

function Shot() {
  return (
    <>
      <Quad z={1.5} />
      <Quad z={0} tint="#ff0000" />
      <Quad z={-1.5} tint="#0000ff" />
      <Camera3d transform={Transform.fromXyz(3, 5, 8).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
