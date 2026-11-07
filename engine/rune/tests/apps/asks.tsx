// R4a's test page for the host bindings (not an R5 app): frames asked for with `ticks`, a host call answered later,
// a synchronous one answered at once, and a timer on the host's clock.
import { useEffect, useState } from "react";
import { call, callSync, emit, on } from "@raea/ui/wire";

export default function Asks() {
  const [frames, setFrames] = useState(0);
  const [answer, setAnswer] = useState("pending");
  const [sync, setSync] = useState("");
  const [timer, setTimer] = useState("waiting");
  useEffect(() => {
    emit("ticks", true);
    const off = on<number>("frame", () => setFrames((f) => f + 1));
    call("nothing").then(() => setAnswer("ok"), (e) => setAnswer(`err: ${e?.message ?? e}`));
    try {
      setSync(`ok: ${callSync("nothing")}`);
    } catch (e) {
      setSync(`err: ${(e as Error).message}`);
    }
    const t = setTimeout(() => setTimer(`fired at ${Math.round(performance.now())}`), 100);
    return () => (off(), clearTimeout(t));
  }, []);
  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      <span data-testid="frames">{frames > 0 ? "ticking" : "still"}</span>
      <span data-testid="answer">{answer}</span>
      <span data-testid="sync">{sync}</span>
      <span data-testid="timer">{timer}</span>
    </div>
  );
}
