// docs/sdk.feature S2d's bench: /ecs's typed query on the host's World (HostWorld), outside React's render, in a plain App
// (bench/bridge/examples/host_query.rs). The App's component is registered for reflection only (no share_component). The
// query's write binding sets the count to 100 once; a Bevy system counts up from there; the text shows the World's
// value as the query answers it, a frame behind.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { component } from "raea/bevy";
import { entity, HostWorld, read, write } from "raea/ecs";
import { useEffect, useState } from "react";

const Hq = component<{ n: number }>("bench::Hq");
const Name = component<string>("bevy_ecs::name::Name");

function Shot() {
  const [shown, setShown] = useState("hq none");
  useEffect(() => {
    const w = new HostWorld();
    let set = false;
    const t = setInterval(() => {
      for (const [, name, hq] of w.query(entity, read(Name), write(Hq))) {
        if (name !== "hq") continue;
        if (!set && hq.n < 100) {
          hq.n = 100;
          set = true;
        }
        setShown(`hq ${hq.n}`);
      }
    }, 4);
    return () => {
      clearInterval(t);
      w.dispose();
    };
  }, []);
  return (
    <AbsoluteFill>
      <div id="hq">{shown}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="hostquery" component={Shot} width={320} height={180} fps={30} durationInFrames={600} />);
