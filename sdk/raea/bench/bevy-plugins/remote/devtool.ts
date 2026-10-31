// bevy_remote as the dev tool (sdk/raea/docs/bevy-plugins.md): an avian App served over BRP's HTTP, so any BRP client
// (curl, an editor, an inspector) can list, query and change it while it runs. The program spawns a ball, then steps
// until a client calls raea/stop; each step answers the requests that came in since the last one. run.sh drives it
// with curl.
//   raea-bevy-sdk motion run sdk/raea/bench/bevy-plugins/remote/devtool.ts      (BRP on port 15703)
import { BevyApp, at, with as w } from "raea/bevy";
import { print } from "raea/native";
import { avian } from "raea/plugins/avian";

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
  const port = 15703;
  const app = BevyApp.open({ plugins: ["avian"], fps: 60, serve: port });
  app.spawn(w(avian.RigidBody, "Dynamic"), w(avian.ColliderConstructor, { Sphere: { radius: 0.5 } }), at(0, 20, 0));
  print(`serving BRP on 127.0.0.1:${port}`);
  let frames = 0;
  // a frame per step; a client's request is answered at the end of the frame it arrived in
  while (!app.call<boolean>("raea/stopped") && frames < 2_000_000) frames = app.step(1);
  print(JSON.stringify({ proof: "remote", frames }));
  app.close();
}
