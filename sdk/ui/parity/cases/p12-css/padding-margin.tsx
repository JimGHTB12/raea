// Padding and margin on each side, with Tailwind's spacing scale and arbitrary values.
// proves: css.properties.padding, css.properties.margin, tw.utility.p-*, tw.utility.m-*
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="outer" className="w-80 bg-slate-200 pt-2 pr-4 pb-6 pl-8">
        <div data-testid="inner" className="mt-3 mr-[7px] mb-1 ml-5 h-10 bg-rose-400" />
        <div data-testid="next" className="m-2 h-6 bg-sky-400" />
      </div>
    </div>
  );
}
