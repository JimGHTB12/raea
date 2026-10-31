// A blurred backdrop fill (P26 anything/edit/sequence): blur(40px) on a box 40 px past each edge, clipped, over ink.
// proves: css.properties.filter
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full" style={{ position: "relative", background: "#0b0b0f", overflow: "hidden" }}>
      <div data-testid="blur" style={{ position: "absolute", left: -40, top: -40, width: 880, height: 680, filter: "blur(40px)", opacity: 0.7, background: "linear-gradient(90deg, #f0c, #fc0 30%, #0cf 60%, #fff)" }} />
    </div>
  );
}
