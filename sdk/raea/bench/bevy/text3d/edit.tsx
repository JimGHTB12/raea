// Text in the 3D world (docs/ts-native.feature P19): "RAEA" animates in letter by letter on a lit stage. Each glyph is
// its own entity (<RichText>, raea-render's text3d.rs): the word rises in with the 2D `pop` and `rise` moves staggered 4 frames
// a letter, each letter spinning up from flat with a spring written in the same code a 2D title uses; the last word,
// a <Span> in another face and colour, types on with `reveal`. A warm spot rakes across it and throws the letters'
// shadows on the floor, a cool rim light from behind, and a pillar passes in front (the text is occluded, not drawn
// over it). Render equals play: sdk/raea/bench/bevy/check.sh sdk/raea/bench/bevy/text3d/edit.tsx
import { AbsoluteFill, Composition, interpolate, registerRoot, spring, useCurrentFrame, useVideoConfig } from "raea";
import { Cylinder, RichText, Span } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight, SpotLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 72;

function Shot() {
  const f = useCurrentFrame();
  const { fps } = useVideoConfig();
  const pillar = interpolate(f, [0, FRAMES - 1], [-4.5, 4.5]);
  return (
    <AbsoluteFill style={{ background: "#07080c" }}>
      <>
        <Camera3d transform={Transform.fromXyz(interpolate(f, [0, FRAMES - 1], [-1.2, 1.2]), 1.6, 6.5).lookingAt([0, 1.1, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={48} />
        <DirectionalLight color="#8fb8ff" transform={Transform.fromXyz(2, 4, -6).lookingAt([0, 0, 0])} illuminance={4800} />
        <SpotLight color="#ffd8a8" transform={Transform.fromXyz(-3.5, 4.5, 3.5).lookingAt([0, 1, 0])} intensity={2700000} outer_angle={0.55} inner_angle={0.275} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)} name="floor">
          <Cuboid half_size={[7, 0.05, 5]} />
          <StandardMaterial base_color="#2a2d34" perceptual_roughness={0.85} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(pillar, 1.2, 1.6)} name="pillar">
          <Cylinder radius={0.22} half_height={1.2} />
          <StandardMaterial base_color="#c0392b" perceptual_roughness={0.5} />
        </Mesh3d>
        <RichText
          name="title"
          position={[0, 1.45, 0]}
          size={1.1}
          font="Montserrat"
          weight={900}
          color="#f4f1ea"
          roughness={0.35}
          metalness={0.2}
          enter={["pop", "rise"]}
          each={4}
          glyph={(g) => {
            const s = spring({ frame: f - g.index * 4, fps, config: { damping: 12 } });
            return { rotation: [(1 - s) * -Math.PI / 2, 0, 0] };
          }}
        >
          RAEA
        </RichText>
        <RichText name="tag" position={[0, 0.35, 0.3]} size={0.32} color="#9fb3c8" reveal={Math.max(0, (f - 24) / 2)}>
          render <Span font="Playfair Display" italic weight={700} color="#ffd400">anything</Span>
        </RichText>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="text3d" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
