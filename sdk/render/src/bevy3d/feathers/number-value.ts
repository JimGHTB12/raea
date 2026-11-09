// bevy_feathers' number_input.rs values (docs/control.feature FC0a): NumberInputValue and NumberInputRange in their
// four number types, and the UnitsFormat a FeathersNumberInput shows its value through. A value prints as Rust's
// Display does ({} on an f32, f64, i32 or i64): the shortest digits that read back as the same number, never in
// exponent form, with no ".0" on a whole number ("1", "0.1", "-0", "NaN", "inf"). A StandardUnitKind scales the
// canonical value into its display unit (an f32 scaled in f64 and cast back, as Bevy does) and appends the unit's
// first suffix.

/** a number input's value, in one of four types. @bevy bevy_feathers::controls::NumberInputValue */
export type NumberInputValue = { F32: number } | { F64: number } | { I32: number } | { I64: number };
/** the number type of a NumberInputValue or NumberInputRange */
export type NumberKind = "F32" | "F64" | "I32" | "I64";

const INT: Record<"I32" | "I64", [number, number]> = { I32: [-(2 ** 31), 2 ** 31 - 1], I64: [Number.MIN_SAFE_INTEGER, Number.MAX_SAFE_INTEGER] };

function num(who: string, kind: NumberKind, v: number): number {
  if (typeof v !== "number") throw new TypeError(`${who}: a ${kind} is a number, got ${JSON.stringify(v)}`);
  if (kind === "F32") return Math.fround(v);
  if (kind === "F64") return v;
  const [lo, hi] = INT[kind];
  if (!Number.isInteger(v) || v < lo || v > hi) throw new RangeError(`${who}: a ${kind} is a whole number from ${lo} to ${hi}, got ${v}`);
  return v;
}

/** NumberInputValue's variants: an F32 is rounded to f32. @bevy bevy_feathers::controls::NumberInputValue */
export const NumberInputValue = {
  F32: (v: number): NumberInputValue => ({ F32: num("NumberInputValue::F32", "F32", v) }),
  F64: (v: number): NumberInputValue => ({ F64: num("NumberInputValue::F64", "F64", v) }),
  I32: (v: number): NumberInputValue => ({ I32: num("NumberInputValue::I32", "I32", v) }),
  I64: (v: number): NumberInputValue => ({ I64: num("NumberInputValue::I64", "I64", v) }),
};

/** an inclusive range of one number type. @bevy bevy_feathers::controls::NumberInputRange */
export type NumberInputRange = { kind: NumberKind; start: number; end: number };

function range(kind: NumberKind, start: number, end: number): NumberInputRange {
  const who = `NumberInputRange::${kind}`;
  return { kind, start: num(who, kind, start), end: num(who, kind, end) };
}

/** NumberInputRange's variants, start..=end. @bevy bevy_feathers::controls::NumberInputRange */
export const NumberInputRange = {
  F32: (start: number, end: number) => range("F32", start, end),
  F64: (start: number, end: number) => range("F64", start, end),
  I32: (start: number, end: number) => range("I32", start, end),
  I64: (start: number, end: number) => range("I64", start, end),
};

/** a NumberInputValue's type and number, checked */
export function unpack(who: string, v: NumberInputValue): [NumberKind, number] {
  const keys = v && typeof v === "object" ? Object.keys(v) : [];
  const kind = keys[0] as NumberKind;
  if (keys.length !== 1 || !(kind === "F32" || kind === "F64" || kind === "I32" || kind === "I64")) {
    throw new TypeError(`${who}: value is a NumberInputValue (NumberInputValue.F32(1)), got ${JSON.stringify(v)}`);
  }
  return [kind, num(who, kind, (v as Record<string, number>)[kind])];
}

function sameKind(who: string, r: NumberInputRange, kind: NumberKind) {
  if (!r || typeof r !== "object" || r.kind !== kind) throw new TypeError(`${who}: a ${kind} value takes a NumberInputRange.${kind}, got ${JSON.stringify(r)}`);
}

/** NumberInputRange::clamp: the value held within the range (f32 min/max, as Rust's clamp) */
export function clamp(who: string, r: NumberInputRange, kind: NumberKind, v: number): number {
  sameKind(who, r, kind);
  return Number.isNaN(v) ? v : Math.min(Math.max(v, r.start), r.end);
}

/** NumberInputRange::thumb_position: where the value sits in the range, 0 to 1 (0.5 on an empty range) */
export function thumb_position(who: string, r: NumberInputRange, kind: NumberKind, v: number): number {
  sameKind(who, r, kind);
  if (!(r.end > r.start)) return 0.5;
  // an f32 range divides in f32; the others in f64, then cast to f32
  if (kind === "F32") return Math.fround(Math.fround(v - r.start) / Math.fround(r.end - r.start));
  return Math.fround((v - r.start) / (r.end - r.start));
}

/** digits and exponent (d.ddd x 10^e) written out in full, as Rust's Display writes a float */
function plain(sci: string): string {
  const [mant, exp] = sci.split("e");
  const neg = mant.startsWith("-");
  const digits = mant.replace("-", "").replace(".", "");
  const e = Number(exp);
  let out: string;
  if (e >= 0) {
    const whole = digits.padEnd(e + 1, "0");
    const frac = whole.slice(e + 1).replace(/0+$/, "");
    out = whole.slice(0, e + 1) + (frac ? `.${frac}` : "");
  } else {
    out = `0.${"0".repeat(-e - 1)}${digits.replace(/0+$/, "")}`;
  }
  return (neg ? "-" : "") + out;
}

function special(v: number): string | null {
  if (Number.isNaN(v)) return "NaN";
  if (v === Infinity) return "inf";
  if (v === -Infinity) return "-inf";
  if (Object.is(v, -0)) return "-0";
  return null;
}

/** Rust's {} on an f32: the fewest significant digits that read back as the same f32 */
export function display_f32(v: number): string {
  const s = special(v);
  if (s !== null) return s;
  for (let p = 1; p <= 9; p++) {
    const sci = v.toExponential(p - 1);
    if (Math.fround(Number(sci)) === v) return plain(sci);
  }
  return plain(v.toExponential(8));
}

/** Rust's {} on an f64: the shortest digits that read back as the same f64 (JavaScript's own) */
export function display_f64(v: number): string {
  return special(v) ?? plain(v.toExponential());
}

/** NumberInputValue's Display */
export function display_value(kind: NumberKind, v: number): string {
  return kind === "F32" ? display_f32(v) : kind === "F64" ? display_f64(v) : String(v);
}

/** formats a number input's value for display (or for editing). @bevy bevy_feathers::controls::UnitsFormat */
export interface UnitsFormat {
  /** the format's name */
  id: string;
  /** the value as text: for display, or for editing when \`editing\` is true */
  format(value: NumberInputValue, editing: boolean): string;
}

/** one unit of a StandardUnitKind: its suffixes (the first canonical) and its size in the base unit.
 *  @bevy bevy_feathers::controls::StandardUnitItem */
export interface StandardUnitItem {
  suffixes: string[];
  scale: number;
}

/** a flat table of units with scale factors, as a UnitsFormat. @bevy bevy_feathers::controls::StandardUnitKind */
export function StandardUnitKind(id: string, units: StandardUnitItem[], canonical_index = 0, display_index = canonical_index, editing_index = display_index): UnitsFormat {
  return {
    id,
    format(value, editing) {
      const [kind, v] = unpack(`units ${id}`, value);
      const shown = units[editing ? editing_index : display_index];
      const scale = units[canonical_index].scale / shown.scale;
      const digits = kind === "F32" ? display_f32(Math.fround(v * scale)) : display_f64(v * scale);
      return digits + shown.suffixes[0];
    },
  };
}

/** no units: the number alone. @bevy bevy_feathers::controls::Dimensionless */
export const Dimensionless: UnitsFormat = { id: "none", format: (value) => display_value(...unpack("units none", value)) };

/** lengths, canonical and shown in metres. @bevy bevy_feathers::controls::LengthMeters */
export const LengthMeters = StandardUnitKind("length_meters", [
  { suffixes: ["m"], scale: 1 }, { suffixes: ["km"], scale: 1000 }, { suffixes: ["cm"], scale: 0.01 }, { suffixes: ["mm"], scale: 0.001 },
  { suffixes: ["ft"], scale: 0.3048 }, { suffixes: ["in"], scale: 0.0254 }, { suffixes: ["mi"], scale: 1609.34 },
]);

/** times, canonical and shown in seconds. @bevy bevy_feathers::controls::TimeSeconds */
export const TimeSeconds = StandardUnitKind("time_seconds", [
  { suffixes: ["s"], scale: 1 }, { suffixes: ["ks"], scale: 1000 }, { suffixes: ["cs"], scale: 0.01 }, { suffixes: ["ms"], scale: 0.001 },
]);

/** angles, canonical in radians, shown in degrees (°, or "d" while editing). @bevy bevy_feathers::controls::AngleDegrees */
export const AngleDegrees = StandardUnitKind("angle_degrees", [
  { suffixes: ["rad"], scale: 1 }, { suffixes: ["°"], scale: Math.PI / 180 }, { suffixes: ["d", "deg"], scale: Math.PI / 180 },
], 0, 1, 2);
