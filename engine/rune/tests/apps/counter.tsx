// R5's counter (projects/rune/js/apps/counter.tsx), as the host's tests run it.
// Text that changes, a button whose style follows state, a reset that appears past 2.
import { useState } from "react";

export default function App() {
  const [n, setN] = useState(0);
  return (
    <div data-testid="root" style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 16, width: 360, height: 240, backgroundColor: "#111318" }}>
      <span data-testid="count" style={{ fontSize: 32, fontWeight: 600, color: "#f2f3f5" }}>{`Count: ${n}`}</span>
      <div style={{ display: "flex", flexDirection: "row", gap: 8 }}>
        <button data-testid="inc" onClick={() => setN(n + 1)} style={{ padding: 10, borderRadius: 8, backgroundColor: n % 2 ? "#3b82f6" : "#2a2d35", color: "#fff", fontSize: 14 }}>
          Add one
        </button>
        {n > 2 && (
          <button data-testid="reset" onClick={() => setN(0)} style={{ padding: 10, borderRadius: 8, backgroundColor: "#2a2d35", color: "#fca5a5", fontSize: 14 }}>
            Reset
          </button>
        )}
      </div>
    </div>
  );
}

export const steps = [{ press: "inc" }, { press: "inc" }, { press: "inc" }, { press: "reset" }, { press: "inc" }];
