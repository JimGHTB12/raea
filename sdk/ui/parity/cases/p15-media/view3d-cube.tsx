// A 3D view of a scene: an unlit 2x2x2 cube seen face on by an orthographic camera 3 units tall, in a 200x150 box, so
// a 100x100 square in the middle; a click widens the box to 300, and the square stays 100x100 in its middle (the
// camera's render target follows the box). Chrome's oracle draws the same with divs.
import { useState } from "react";
import { side } from "parity";
export const steps = [{ click: "wide" }];
export default function Case() {
  const [w, setW] = useState(200);
  const bevy = side === "bevy";
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      {bevy ? (
        <view3d data-testid="view" style={{ width: w, height: 150, flexShrink: 0 }} clearColor="#1e293b" projection="orthographic" viewHeight={3} eye={[0, 0, 10]} target={[0, 0, 0]}>
          <mesh3d shape="cuboid" size={[2, 2, 2]} color="#e11d48" unlit position={[0, 0, 0]} />
        </view3d>
      ) : (
        <div data-testid="view" className="flex items-center justify-center" style={{ width: w, height: 150, flexShrink: 0, backgroundColor: "#1e293b" }}>
          <div style={{ width: 100, height: 100, backgroundColor: "#e11d48" }} />
        </div>
      )}
      <button data-testid="wide" className="h-8 w-20 bg-slate-800" onClick={() => setW(300)} />
    </div>
  );
}
