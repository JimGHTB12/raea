// The badge: the conformance composition's React component, used twice (two uses, two keys).
export function Badge({ x, y, color }: { x: number; y: number; color: string }) {
  return (
    <div style={{ position: "absolute", left: x, top: y, width: 90, height: 90, borderRadius: 18, background: color }}>
      <span style={{ position: "absolute", left: 12, top: 30, color: "#fff", fontSize: 22, fontWeight: 700 }}>badge</span>
    </div>
  );
}
