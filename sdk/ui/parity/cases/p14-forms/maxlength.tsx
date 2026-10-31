// maxLength: typing past it is cut off.
// proves: html.elements.input.maxlength
export const steps = [{ click: "f" }, { text: "abcdefgh" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="f" className="h-9 w-56 border border-slate-400 px-2 text-sm" maxLength={5} />
    </div>
  );
}
