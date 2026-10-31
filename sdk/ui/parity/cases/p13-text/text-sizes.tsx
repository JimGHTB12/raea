// Tailwind's text sizes, each with its own line height.
// proves: tw.utility.text-*, css.properties.font-size, css.properties.line-height
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="xs" className="text-xs">size xs</p>
      <p data-testid="sm" className="text-sm">size sm</p>
      <p data-testid="lg" className="text-lg">size lg</p>
      <p data-testid="xl2" className="text-2xl">size 2xl</p>
      <p data-testid="arb" className="text-[13px]">size 13px</p>
    </div>
  );
}
