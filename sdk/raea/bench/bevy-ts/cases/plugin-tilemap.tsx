// gap: PL4
// bevy_ecs_tilemap through raea/plugins/tilemap (docs/plugins.feature PL4): a 2 x 2 map of 32 px tiles from
// plugin-tiles.png (index 0 red, 1 green), anchored at its top left on the 2D overlay (pixels from the frame's top
// left, +y up), its four tiles separate entities naming the map by its part name. Tile y counts up from the bottom
// row: red at the top left and bottom right, green at the other two.
// build: plugins
// frame: 20
// probe: 16 16 r>200 g<60 b<60
// probe: 48 16 g>200 r<60 b<60
// probe: 16 48 g>200 r<60 b<60
// probe: 48 48 r>200 g<60 b<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { TilemapAnchor, TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilePos, TileTextureIndex } from "raea/plugins/tilemap";

const tiles: [number, number, number][] = [[0, 1, 0], [1, 1, 1], [0, 0, 1], [1, 0, 0]];

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity name="map" position={[0, 0, 0]}>
          <BevyComponent type="raea_sys_sprite::Draws2d" value={{}} />
          <TilemapTexture value={{ Single: "plugin-tiles.png" }} />
          <TilemapSize value={{ x: 2, y: 2 }} />
          <TilemapTileSize value={{ x: 32, y: 32 }} />
          <TilemapGridSize value={{ x: 32, y: 32 }} />
          <TilemapAnchor value="TopLeft" />
        </Entity>
        {tiles.map(([x, y, i]) => (
          <Entity key={`${x}${y}`}>
            <TilePos value={{ x, y }} />
            <TilemapId value="map" />
            <TileTextureIndex value={i} />
          </Entity>
        ))}
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={21} fps={30} width={64} height={64} />);
