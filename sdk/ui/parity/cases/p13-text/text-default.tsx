// Text with no size or colour of its own: the page's defaults (16 px, the root's colour), one line in a block.
// proves: react-dom.element.p, css.properties.font-size, css.properties.color
// tier: quick
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="p">Hello parity</p>
    </div>
  );
}
