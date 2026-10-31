// box-sizing: content-box (CSS's initial value, and Tailwind's box-content) puts padding and border outside the width
// and height; border-box (preflight's, and box-border) keeps them inside. Set by class, inline style and a sheet.
// proves: css.properties.box-sizing, css.properties.box-sizing.border-box, css.properties.box-sizing.content-box, tw.utility.box-border, tw.utility.box-content
export const steps = [];
const CSS = `.sheet-content { box-sizing: content-box }`;
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <style>{CSS}</style>
      <div data-testid="border" className="box-border h-16 w-40 border-4 border-slate-700 bg-sky-300 p-4" />
      <div data-testid="content" className="mt-4 box-content h-16 w-40 border-4 border-slate-700 bg-amber-300 p-4" />
      <div data-testid="inline" className="mt-4 h-16 w-40 border-4 border-slate-700 bg-emerald-300 p-4" style={{ boxSizing: "content-box" }} />
      <div data-testid="sheet" className="sheet-content mt-4 h-16 w-40 border-2 border-slate-700 bg-rose-300 px-6 py-2" />
      <div data-testid="row" className="mt-4 flex gap-2">
        <div data-testid="grow" className="box-content flex-1 border-4 border-slate-700 bg-violet-300 p-2">a</div>
        <div data-testid="fixed" className="box-content w-24 bg-slate-300 px-8">b</div>
      </div>
    </div>
  );
}
