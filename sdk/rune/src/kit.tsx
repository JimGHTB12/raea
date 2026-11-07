// rune-kit's components in TSX (R6a). Each renders a plain DOM stand-in that a browser or Blitz draws as it is, and
// carries `data-rune-kit` with what the Rune host needs to build the real component in its place (host/src/kit.rs).
// Presses on an interactive one still reach React: the host sends them to the page as the browser would.
import type { ReactElement } from "react";

type Kit = { kit: string } & Record<string, unknown>;
const kit = (k: Kit) => ({ "data-rune-kit": JSON.stringify(k) });

/** Which neutral kit draws the component in the Rune host: the light one (default) or the dark one, for dark surfaces. */
type Theme = { dark?: boolean };

/** A small label: neutral, accent or subtle. */
export function Badge({ label, variant = "neutral", dark }: { label: string; variant?: "neutral" | "accent" | "subtle" } & Theme): ReactElement {
  return <span {...kit({ kit: "badge", label, variant, dark })}>{label}</span>;
}

/** An indeterminate spinner. */
export function Spinner({ dark }: Theme = {}): ReactElement {
  return <span role="progressbar" aria-busy="true" {...kit({ kit: "spinner", dark })} />;
}

/** A progress bar: `value` from 0 to 1, or none for indeterminate. */
export function Progress({ value, dark }: { value?: number } & Theme): ReactElement {
  return <progress value={value} max={1} {...kit({ kit: "progress", value: value ?? null, dark })} />;
}

/** Text whose characters roll to their new values when it changes (a counter, a price). */
export function RollingText({ text, motion }: { text: string; motion?: import("./index.ts").Motion }): ReactElement {
  return <span {...kit({ kit: "rolling", text, motion: motion ?? null })}>{text}</span>;
}

/** An on/off switch; `onChange` hears the new value. */
export function Switch({ checked, onChange, label, dark }: { checked: boolean; onChange: (next: boolean) => void; label?: string } & Theme): ReactElement {
  return (
    <button role="switch" aria-checked={checked} aria-label={label} onClick={() => onChange(!checked)} {...kit({ kit: "switch", checked, label: label ?? null, dark })}>
      {label}
    </button>
  );
}
