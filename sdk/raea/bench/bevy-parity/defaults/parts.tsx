// The defaults test's scene (docs/control.feature FC0c): each Bevy-named part written with no props but what it can't
// go without, so the components raea gives them can be held against Bevy's own Default (tools/bevy-defaults).
import { Composition, registerRoot } from "raea";

import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { AmbientLight, DirectionalLight, PointLight, SpotLight } from "raea/light";

function Shot() {
  return (
    <>
      <Camera3d />
      <AmbientLight />
      <DirectionalLight />
      <PointLight />
      <SpotLight />
      <Mesh3d>
        <Cuboid />
        <StandardMaterial />
      </Mesh3d>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={1} fps={30} width={320} height={180} />);
