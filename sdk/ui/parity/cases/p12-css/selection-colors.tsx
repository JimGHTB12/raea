// ::selection colours: a word double clicked in light text on a dark box takes the box's ::selection background and
// colour (Tailwind's selection: variant, as apps/app's index.css sets it), on the word in a bold span too.
// proves: css.selectors.selection, tw.variant.selection
export const steps = [{ click: "ed", at: [0.12, 0.5], clicks: 2 }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-zinc-900 p-4 text-zinc-100">
      <div
        data-testid="ed"
        contentEditable
        suppressContentEditableWarning
        className="w-80 p-2 text-lg outline-none selection:bg-indigo-400/40 selection:text-amber-200"
      >
        Fieldwork <b>Creative</b> handbook
      </div>
    </div>
  );
}
