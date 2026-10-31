// A stylesheet's :root custom properties and its own classes (a brand kit beside the Tailwind build): classes Tailwind
// doesn't define read var(--brand-*), calc() works out sizes, and an inline style reads a variable too.
// proves: css.properties.custom-property, css.types.var, css.types.calc, css.selectors.class
export const steps = [];
const css = `:root { --brand-accent: #e11d48; --brand-surface: #0f172a; --brand-pad: 12px; --brand-size: 14px; }
.bg-brand-surface { background-color: var(--brand-surface); }
.text-brand-accent { color: var(--brand-accent); }
.p-brand { padding: var(--brand-pad); }
.text-brand-title { font-size: calc(var(--brand-size) * 2); line-height: 1.25; }
.w-brand { width: calc(50% - var(--brand-pad)); }`;
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <style>{css}</style>
      <div data-testid="card" className="bg-brand-surface p-brand w-brand">
        <p data-testid="title" className="text-brand-accent text-brand-title">Title</p>
      </div>
      <div data-testid="inline" style={{ height: 20, width: "calc(var(--brand-pad) * 10)", backgroundColor: "var(--brand-accent)" }} />
    </div>
  );
}
