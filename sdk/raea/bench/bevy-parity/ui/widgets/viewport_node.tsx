// Bevy v0.20.0 examples/ui/widgets/viewport_node.rs in raea's TSX (docs/control.feature FC0a): a 5 m white cube at
// z -10, unlit but for the default ambient light, seen by two default Camera3ds at the origin: the window's, and one
// (order -1) drawing into an image a ViewportNode shows, an absolute 200 x 200 node at (50, 50) with a 5 px white
// border. The ViewportNode sizes the image to the node (computed_node.size(): its border box, 200 x 200) and the
// camera's aspect with it, and the border is drawn over the image. Here that camera draws into its own viewport
// of the frame, over the window camera's picture (order 1, its own
// clear, as the image clears), and the bordered node is drawn over both. Dragging the cube or the node (and the
// intersection arrows MeshPickingPlugin draws) needs a pointer, which no render has. In Bevy's default 1280 x 720
// window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";
import { Node, UiRect, px } from "raea/ui";
import { Srgba } from "raea/color";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Camera3d transform={Transform.IDENTITY} />
        <Entity name="viewport-camera" position={[0, 0, 0]}>
          <BevyComponent type="bevy_camera::components::Camera3d" value={{}} />
          <BevyComponent type="bevy_camera::camera::Camera" value={{ order: 1, viewport: { physical_position: [50, 50], physical_size: [200, 200], depth: { start: 0, end: 1 } } }} />
        </Entity>
        <Mesh3d transform={Transform.fromXyz(0, 0, -10)}>
          <Cuboid half_size={[2.5, 2.5, 2.5]} />
          <StandardMaterial base_color="#ffffff" />
        </Mesh3d>
      </Canvas3D>
      <Node position_type="Absolute" top={px(50)} left={px(50)} width={px(200)} height={px(200)} border={UiRect.all(px(5))} border_color={new Srgba(1, 1, 1)} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
