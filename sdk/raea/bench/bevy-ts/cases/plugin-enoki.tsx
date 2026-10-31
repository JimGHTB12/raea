// gap: PL8
// bevy_enoki through raea/plugins/enoki (docs/plugins.feature PL8): Sparks, raea's reflected spawner over enoki's
// ParticleSpawner and Particle2dEffect, on an entity 48 px down the frame's centre line in the 2D overlay (pixels
// from the top left, +y up). With no spread every particle rises straight up at 60 px/s, so by frame 20 (0.67 s)
// they make an orange column (linear 1, 0.6, 0.2: sRGB 255, 203, 124) from the spawner up to about 8 px from the top; the frame's sides stay black.
// build: plugins
// frame: 20
// probe: 32 30 r>200 g>150 b<170
// probe: 4 30 r<20 g<20 b<20
// probe: 60 30 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { Sparks } from "raea/plugins/enoki";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[32, -48, 0]}>
          <Sparks value={{ every: 0.02, amount: 1, lifetime: 1, speed: 60, direction: [0, 1], spread: 0, size: 6, color: [1, 0.6, 0.2, 1], gravity: 0 }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={24} fps={30} width={64} height={64} />);
