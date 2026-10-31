// The background shorthand in an inline style: a plain colour, a gradient, and a colour under a gradient.
// proves: css.properties.background, css.properties.background-color, css.properties.background-image
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full p-4" style={{ background: "#101010" }}>
      <div data-testid="plain" className="mb-2 h-12 w-40" style={{ background: "#f2c14e" }} />
      <div data-testid="gradient" className="mb-2 h-12 w-40" style={{ background: "linear-gradient(90deg, #e11d48, #2563eb)" }} />
      <div data-testid="both" className="h-12 w-40" style={{ background: "rgb(14, 22, 32) linear-gradient(180deg, rgba(255,255,255,0.5), rgba(255,255,255,0))" }} />
    </div>
  );
}
