// gap: PL4
// bevy_ecs_tiled through raea/plugins/tiled (docs/plugins.feature PL4): a 2 x 2 Tiled map of 32 px tiles
// (plugin-tiled.tmx, its tileset plugin-tiles.png: a red tile and a green one) loaded by its TiledMap, anchored at its
// top left on the 2D overlay's (pixels from the frame's top left, +y up): red at the top left and bottom right, green
// at the other two.
// build: plugins
// frame: 20
// probe: 16 16 r>200 g<60 b<60
// probe: 48 16 g>200 r<60 b<60
// probe: 16 48 g>200 r<60 b<60
// probe: 48 48 r>200 g<60 b<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { TiledMap } from "raea/plugins/tiled";
import { TilemapAnchor } from "raea/plugins/tilemap";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[0, 0, 0]}>
          <BevyComponent type="raea_sys_sprite::Draws2d" value={{}} />
          <TiledMap value="plugin-tiled.tmx" />
          <TilemapAnchor value="TopLeft" />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={21} fps={30} width={64} height={64} />);
