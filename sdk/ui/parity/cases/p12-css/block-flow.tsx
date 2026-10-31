// Normal flow: plain divs stack top to bottom, each as wide as its parent (display: block, not flex).
// proves: css.properties.display, tw.utility.block
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="a" className="h-10 bg-rose-300" />
      <div data-testid="b" className="h-14 bg-sky-300" />
      <div data-testid="c" className="h-6 w-1/2 bg-emerald-300" />
    </div>
  );
}
