// Cinema fixture: a dark toon room lit through a window, with the camera's effects together: volumetric fog shows the
// window's shaft, depth of field pulls focus from the back wall (12) onto the hero (6.1 by frame 45), a grade warms
// the picture and ambient occlusion darkens the contacts, all on throughout; MotionBlur smears the ball rolling across
// from frame 20 (frames 0-19 have no MotionBlur). Each effect's lines are marked {/* fx:NAME */} so cinema/check.py can
// take them out; render equals its baked play (check.sh).
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { AmbientOcclusion, DepthOfField, FogVolume, Grade, MotionBlur, Sphere, VolumetricFog, manifold } from "raea/3d";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Quat, Transform } from "raea/transform";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Camera3d } from "raea/camera";
import { ExtendedMaterial, StandardMaterial, ToonExtension } from "raea/pbr";

const FPS = 30;
const FRAMES = 60;
const WALL = manifold.box(0.3, 6, 14).minus(manifold.box(1, 2.6, 2.4).move(0, 0.6, -0.5));

function Room() {
  const f = useCurrentFrame();
  const focus = interpolate(f, [20, 45], [12, 6.1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ background: "#05060a" }}>
      <>
        <Camera3d transform={Transform.fromXyz(1.5, 2, 7).lookingAt([0, 1.8, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }}><DepthOfField focal_distance={focus} aperture_f_stops={0.06} /></Camera3d>
        <AmbientLight brightness={60} />
        <DirectionalLight color="#ffe2b0" volumetric={true} transform={Transform.fromXyz(-12, 9, -2).lookingAt([0, 0, 0])} illuminance={20000} shadow_maps_enabled />
        <VolumetricFog />{/* fx:vfog */}
        <FogVolume position={[0, 3, 0]} scale={[10, 6.5, 14]} density={0.25} anisotropy={0.8} />{/* fx:vfog */}
        {/* fx:dof */}
        <Grade temperature={0.3} saturation={1.15} />{/* fx:grade */}
        {f >= 20 ? <MotionBlur shutterAngle={1} /> : null}{/* fx:mblur */}
        <AmbientOcclusion />{/* fx:ao */}
        <Mesh3d transform={Transform.fromXyz(-4, 3, 0)} name="wallL" shape={WALL}><ExtendedMaterial base={<StandardMaterial base_color="#5b6378" />} extension={<ToonExtension steps={2} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 3, -7)} name="wallB"><Cuboid half_size={[4, 3, 0.15]} /><ExtendedMaterial base={<StandardMaterial base_color="#4a5066" />} extension={<ToonExtension steps={2} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, -0.15, 0)} name="floor"><Cuboid half_size={[4, 0.15, 7]} /><ExtendedMaterial base={<StandardMaterial base_color="#8a6a52" />} extension={<ToonExtension steps={2} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 6.15, 0)} name="ceil"><Cuboid half_size={[4, 0.15, 7]} /><ExtendedMaterial base={<StandardMaterial base_color="#3a3f50" />} extension={<ToonExtension steps={2} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0.5, 1.1, 1)} name="hero"><Sphere radius={0.9} /><ExtendedMaterial base={<StandardMaterial base_color="#ffd23f" />} extension={<ToonExtension steps={2} rim={{ intensity: 0.6, width: 0.2 }} outline={{ width: 3 }} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.5, 0.6, -3.5).withRotation(Quat.fromEuler("XYZ", 0, 0.4, 0))} name="crate"><Cuboid half_size={[0.6, 0.6, 0.6]} /><ExtendedMaterial base={<StandardMaterial base_color="#c47a3a" />} extension={<ToonExtension steps={2} outline={{ width: 3 }} />} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(3.4 - f * 0.1, 0.5, 3)} name="ball"><Sphere radius={0.5} /><ExtendedMaterial base={<StandardMaterial base_color="#3aa0ff" />} extension={<ToonExtension steps={2} outline={{ width: 3 }} />} /></Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="cinema" component={Room} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
