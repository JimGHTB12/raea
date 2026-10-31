// A scroll container: overflow-y-auto with more content than fits, scrolled by the wheel.
// proves: css.properties.overflow-y, tw.utility.overflow-y-auto, api.Element.scrollTop
// tier: quick
export const steps = [{ wheel: "list", dy: 120 }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="list" className="h-40 w-64 overflow-y-auto bg-slate-100">
        {Array.from({ length: 12 }, (_, i) => <div key={i} data-testid={"r" + i} className="h-10 border-b border-slate-300 bg-white" />)}
      </div>
    </div>
  );
}
