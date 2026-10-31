// The style prop: sizes in numbers (px), colours, padding and a percentage width.
// proves: react-dom.prop.style, css.properties.background-color
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div data-testid="a" style={{ width: 180, height: 40, backgroundColor: "#0ea5e9", marginBottom: 8 }} />
      <div data-testid="b" style={{ width: "50%", height: 30, backgroundColor: "rgb(244, 63, 94)", padding: 6 }}>
        <div data-testid="c" style={{ height: 10, backgroundColor: "#111827" }} />
      </div>
    </div>
  );
}
