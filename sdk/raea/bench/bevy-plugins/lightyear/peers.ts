// lightyear from a TypeScript program (sdk/raea/docs/bevy-plugins.md): two headless peers in one process, a server
// App and a client App, linked by lightyear's in-process crossbeam transport and stepped in lockstep. The program
// starts the server, links and connects the client, spawns a player the client controls, sends it input, and checks
// these things through BRP and the adapter's lightyear/status:
// - the replicated Position moves on both peers;
// - the client's predicted copy runs ahead of the server while the input is held;
// - the two agree once the input stops;
// - the session state (Lobby, then Playing, set from TypeScript) gates the input;
// - a Teleport event triggered on the server is observed, replicated and corrects the client's prediction;
// - an Announce message is read into the Announced resource.
// The session is a function of its frames: the peers' clock is lightyear's mock Instant, advanced once per frame.
//   raea-bevy-sdk motion run sdk/raea/bench/bevy-plugins/lightyear/peers.ts      (run.sh runs it)
import { BevyApp } from "raea/bevy";
import { print } from "raea/native";
import { lightyear } from "raea/plugins/lightyear";

interface Status {
  connected: boolean | number;
  tick: number;
  entities: { entity: number; position: [number, number, number]; predicted: boolean; controlled: boolean }[];
}

function check(ok: boolean, what: string) {
  if (!ok) {
    // printed too: a thrown Error's stack in JavaScriptCore leaves its message out
    print(`FAIL lightyear peers: ${what}`);
    throw new Error(`lightyear peers: ${what}`);
  }
}

function until(peers: BevyApp[], done: () => boolean, most: number, what: string): number {
  for (let f = 0; f < most; f++) {
    if (done()) return f;
    BevyApp.stepTogether(peers, 1);
  }
  throw new Error(`lightyear peers: ${what} within ${most} frames`);
}

export default function main() {
  // a throw's message printed: JavaScriptCore's stack leaves it out
  try {
    run();
  } catch (e) {
    print(`FAIL: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}

function run() {
  const server = BevyApp.open({ plugins: ["lightyear_server"], fps: 60 });
  const client = BevyApp.open({ plugins: ["lightyear_client"], fps: 60 });
  // the client updates before the server each frame, as lightyear's own test stepper orders them
  const peers = [client, server];
  server.call("lightyear/start");
  const { link } = server.call<{ link: number }>("lightyear/link");
  client.call("lightyear/connect", { link });
  const connectedAt = until(peers, () => client.call<Status>("lightyear/status").connected === true && server.call<Status>("lightyear/status").connected === 1, 200, "the client didn't connect");
  const player = server.call<number>("lightyear/spawn", { position: [0, 0, 0], owner: link });
  const predictedAt = until(peers, () => client.call<Status>("lightyear/status").entities.some((e) => e.predicted), 200, "the player wasn't replicated to the client as predicted");
  // input is ignored in the Lobby: held for 10 frames, nothing moves
  client.call("lightyear/input", { x: 1, z: 0 });
  BevyApp.stepTogether(peers, 10);
  const lobby = server.call<Status>("lightyear/status").entities.find((e) => e.entity === player)!.position;
  check(lobby[0] === 0, `in the Lobby the player shouldn't move, it's at ${lobby}`);
  // the state, set on both peers from TypeScript (raea/state.set), changes at the next frame's transition
  for (const p of peers) p.setState(lightyear.Session, "Playing");
  BevyApp.stepTogether(peers, 1);
  check(server.state(lightyear.Session) === "Playing" && client.state(lightyear.Session) === "Playing", `both peers should be Playing, they're ${server.state(lightyear.Session)} and ${client.state(lightyear.Session)}`);
  // hold +x for a second
  BevyApp.stepTogether(peers, 60);
  const held = { client: client.call<Status>("lightyear/status"), server: server.call<Status>("lightyear/status") };
  const c = held.client.entities.find((e) => e.predicted)!;
  const s = held.server.entities.find((e) => e.entity === player)!;
  check(s.position[0] > 0.5, `the server's player should have moved +x under the input, it's at ${s.position}`);
  check(c.position[0] > 0.5, `the client's predicted player should have moved +x, it's at ${c.position}`);
  check(c.position[0] > s.position[0], `the predicted copy should run ahead of the server's (client ${c.position[0]}, server ${s.position[0]})`);
  // let go, and let the server catch up
  client.call("lightyear/input", { x: 0, z: 0 });
  BevyApp.stepTogether(peers, 60);
  const after = { client: client.call<Status>("lightyear/status"), server: server.call<Status>("lightyear/status") };
  const c2 = after.client.entities.find((e) => e.predicted)!;
  const s2 = after.server.entities.find((e) => e.entity === player)!;
  check(Math.abs(c2.position[0] - s2.position[0]) < 1e-3, `after the input stops the peers should agree (client ${c2.position[0]}, server ${s2.position[0]})`);
  // an event and a message, typed by the generated handles: a teleport on the server (observed), a line announced
  const teleports = server.observe(lightyear.Teleport);
  server.trigger(lightyear.Teleport, { to: [5, 0, -2] });
  server.write(lightyear.Announce, { text: "teleported" });
  BevyApp.stepTogether(peers, 60);
  const seen = teleports();
  check(seen.length === 1 && seen[0].to[0] === 5, `the teleport should have been observed once, saw ${JSON.stringify(seen)}`);
  check(server.resource(lightyear.Announced).includes("teleported"), `Announce should have been read, the log is ${JSON.stringify(server.resource(lightyear.Announced))}`);
  const moved = { client: client.call<Status>("lightyear/status").entities.find((e) => e.predicted)!.position, server: server.call<Status>("lightyear/status").entities.find((e) => e.entity === player)!.position };
  check(Math.abs(moved.server[0] - 5) < 1e-4 && Math.abs(moved.client[0] - 5) < 1e-3, `the teleport should replicate to the client and correct its prediction: ${JSON.stringify(moved)}`);
  // the same component through BRP's generic query, typed by the generated handle
  const q = client.query({ data: [lightyear.Position] });
  check(q.length >= 1, "BRP's world.query should see the replicated Position on the client");
  print(JSON.stringify({ proof: "lightyear", connectedAt, predictedAt, held: { client: c.position, server: s.position, ticks: [held.client.tick, held.server.tick] }, after: { client: c2.position, server: s2.position }, teleported: moved, observed: seen, announced: server.resource(lightyear.Announced), query: q }));
  client.call("lightyear/disconnect");
  BevyApp.stepTogether(peers, 5);
  client.close();
  server.close();
}
