// Bevy v0.20.0 examples/3d/spherical_area_lights.rs in raea's TSX (docs/control.feature FC0a): six blue point lights
// along X, each in an unlit sphere of its radius (0 to 0.33 m), over a shiny grey floor whose highlights grow with
// the radius (a spherical area light: PointLight's radius); GlobalAmbientLight at 60.
import { Composition, registerRoot } from "raea";
import { Fragment } from "react";
import { Camera3d } from "raea/camera";
import { GlobalAmbientLight, PointLight } from "raea/light";
import { Mesh3d, Plane3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const COUNT = 6;

function Shot() {
  return (
    <>
      <GlobalAmbientLight brightness={60} />
      <Camera3d transform={Transform.fromXyz(0.2, 1.5, 2.5).lookingAt([0, 0, 0])} />
      <Mesh3d>
        <Plane3d half_size={[50, 50]} />
        <StandardMaterial base_color="rgb(51, 51, 51)" perceptual_roughness={0.08} />
      </Mesh3d>
      {Array.from({ length: COUNT }, (_, i) => {
        const percent = Math.fround(i / COUNT);
        const radius = Math.fround(percent * 0.4);
        const x = Math.fround(-2 + Math.fround(percent * 4));
        return (
          // the light is the sphere's child in Bevy, at its centre (the sphere's scale doesn't size a light)
          <Fragment key={i}>
            <Mesh3d transform={{ translation: [x, 0.3, 0], scale: [radius, radius, radius] }}>
              <Sphere radius={1} uv={[120, 64]} />
              <StandardMaterial base_color="rgb(127.5, 127.5, 255)" unlit />
            </Mesh3d>
            <PointLight radius={radius} color="rgb(51, 51, 255)" transform={Transform.fromXyz(x, 0.3, 0)} />
          </Fragment>
        );
      })}
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
