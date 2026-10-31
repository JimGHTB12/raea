// line-height in each form React's style object takes, on 32px text: a px string ("44px", once sent to the host as a
// bare 44 and so drawn 44 times the font size, the line far below its box), a unitless number (1.5), a unitless string
// ("1.25"), an em string and a percentage. Each box's height is its line-height.
// proves: css.properties.line-height
export const steps = [];
const FORMS: [string, string | number][] = [["px", "44px"], ["number", 1.5], ["string", "1.25"], ["em", "1.75em"], ["percent", "90%"]];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col items-start gap-2 bg-white p-4">
      {FORMS.map(([name, lh]) => (
        <div key={name} data-testid={name} style={{ fontSize: 32, lineHeight: lh, background: "#e2e8f0", color: "#0f172a" }}>
          line-height {String(lh)}
        </div>
      ))}
    </div>
  );
}
