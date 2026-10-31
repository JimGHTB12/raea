// Studio fixture: a banana on a white turntable in a white cyc, as a product shot. One large soft key (a sun with
// soft shadows, PCSS), image-based fill from an HDRI, ambient occlusion at the contacts, high-key exposure (no tone
// curve, exposure -2.55: the cyc reads #f7f7f5 near the camera and white towards the horizon while the yellow stays
// saturated). The Environment's intensity is 0 to frame 9 and 1 from frame 20. studio/check.py takes features out
// by the markers fx:NAME and measures each; render equals its baked play (check.sh).
// media/studio_small_09_1k.hdr: Poly Haven, "Studio Small 09" by Sergej Majboroda,
// https://polyhaven.com/a/studio_small_09 (dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/studio_small_09_1k.hdr),
// CC0 1.0, downloaded 2026-10-19.
import { AbsoluteFill, Composition, interpolate, registerRoot, staticFile, useCurrentFrame } from "raea";
import { AmbientOcclusion, Cylinder, DirectionalLight, Environment, Shadows, ToneMapping, manifold, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const SOFT = true; // fx:soft
const BANANA = manifold.sphere(1).smooth({ refine: 4 }).scale(1.1, 0.27, 0.27).refineToLength(0.05).warp((x, y, z) => {
  const t = x / 1.1;
  const pinch = 1 - 0.35 * t * t * t * t;
  return [x, y * pinch + 0.45 * t * t, z * pinch];
}).union(manifold.cylinder(0.05, 0.25).rotate(0, 90, 0).rotate(0, 0, 35).move(1.08, 0.5, 0));
// speckles for the peel: an SVG drawn once
const PEEL = "data:image/svg+xml;utf8," + encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512"><rect width="512" height="512" fill="#f4cf3a"/>${Array.from({ length: 70 }, (_, i) => `<circle cx="${(i * 73) % 512}" cy="${(i * 151) % 512}" r="${2 + (i % 5)}" fill="#6b4a1e" opacity="0.55"/>`).join("")}<rect y="0" width="512" height="40" fill="#b8c24a" opacity="0.5"/></svg>`);

function Studio() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#ffffff" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.6, 5.2).lookingAt([0, 0.45, 0])} projection={{ Perspective: { fov: 0.558505360638185 } }} />
        <ToneMapping mode="none" exposure={-2.55} />
        <Shadows bias={0.2} distance={30} mapSize={4096} />
        <DirectionalLight position={[-3, 6, 3]} intensity={2.2} softShadow={SOFT ? 10 : false} />
        <Environment src={staticFile("studio_small_09_1k.hdr")} intensity={interpolate(f, [9, 20], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })} rotation={0.6} />{/* fx:env */}
        <AmbientOcclusion quality="high" thickness={0.15} />{/* fx:ao */}
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)} name="floor"><Cuboid half_size={[100, 0.05, 100]} /><StandardMaterial base_color="#f7f7f5" perceptual_roughness={1} reflectance={0} /></Mesh3d>
        <Entity transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", 0, f * 0.025, 0))}>
          <Mesh3d transform={Transform.fromXyz(0, 0.06, 0)} name="susan"><Cylinder radius={1.25} half_height={0.06} /><StandardMaterial base_color="#f4f4f2" perceptual_roughness={0.35} /></Mesh3d>
          <Mesh3d transform={Transform.fromXyz(0, 0.38, 0).withRotation(Quat.fromEuler("XYZ", 0.15, 0.6, 0.1)).withScale(0.95)} name="banana" shape={BANANA}>
            <StandardMaterial base_color="#ffffff" perceptual_roughness={0.55} base_color_texture={PEEL} />
          </Mesh3d>
        </Entity>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="studio" component={Studio} width={960} height={540} fps={30} durationInFrames={60} />);
