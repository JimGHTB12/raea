// The cross-side calibration: one solid box at a fixed place and size on a full white root, no text. Identical by
// construction on any host that lays out a fixed box, so it must pass both ways; if it doesn't, the harness is wrong
// (coordinates, scale, colour space, the readback), not the host.
// expect: pass
// tier: quick
export const steps = [{ wait: 2 }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white">
      <div data-testid="box" style={{ width: 120, height: 60, backgroundColor: "rgb(225, 29, 72)" }} />
    </div>
  );
}
