// A 3D view's scene from its children, moved by React: two unlit shapes (a cuboid and a sphere) under an orthographic
// camera, the cuboid moved right and recoloured by a click. Chrome's oracle draws the same with divs (the sphere seen
// head on is a disc).
import { useState } from "react";
import { side } from "parity";
export const steps = [{ click: "move" }];
export default function Case() {
  const [moved, setMoved] = useState(false);
  const bevy = side === "bevy";
  // 50 px a unit: the view is 300x200 and the camera 4 units tall
  const px = (x: number) => 150 + x * 50;
  const py = (y: number) => 100 - y * 50;
  const cx = moved ? 1.5 : -1.5;
  const color = moved ? "#16a34a" : "#e11d48";
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      {bevy ? (
        <view3d data-testid="view" style={{ width: 300, height: 200, flexShrink: 0 }} clearColor="#f1f5f9" projection="orthographic" viewHeight={4} eye={[0, 0, 10]} target={[0, 0, 0]}>
          <mesh3d shape="cuboid" size={[1, 1, 1]} color={color} unlit position={[cx, 0.5, 0]} />
          <mesh3d shape="sphere" size={[0.5, 0.5, 0.5]} color="#2563eb" unlit position={[0, -1, 0]} />
        </view3d>
      ) : (
        <div data-testid="view" className="relative" style={{ width: 300, height: 200, flexShrink: 0, backgroundColor: "#f1f5f9" }}>
          <div className="absolute" style={{ left: px(cx) - 25, top: py(0.5) - 25, width: 50, height: 50, backgroundColor: color }} />
          <div className="absolute rounded-full" style={{ left: px(0) - 25, top: py(-1) - 25, width: 50, height: 50, backgroundColor: "#2563eb" }} />
        </div>
      )}
      <button data-testid="move" className="h-8 w-20 bg-slate-800" onClick={() => setMoved(true)} />
    </div>
  );
}
