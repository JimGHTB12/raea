// gap: G19
// A live game's window settings (raea/window, Bevy's Window as the game sets it): set at load, title and a
// locked cursor; play applies them to its window (offscreen there's none, and `play --json` reports them). Green once
// the settings read back as set.
// play: live-world.json 0.2
// probe: 32 32 g>150 r<100
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { gameWindow } from "raea/window";

gameWindow.set({ title: "Dungeon", cursor: "locked" });
gameWindow.set({ resizable: false });

function Shot() {
  const w = gameWindow.get();
  const ok = w.title === "Dungeon" && w.cursor === "locked" && w.resizable === false;
  return <AbsoluteFill style={{ background: ok ? "#00ff00" : "#ff0000" }} />;
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
