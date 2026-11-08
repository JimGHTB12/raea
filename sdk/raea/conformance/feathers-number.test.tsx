// docs/control.feature FC0a: bevy_feathers' FeathersNumberInput (ui/widgets/feathers_number_input). Its text is the
// value as Rust's Display prints it, held within its HardLimit and shown through its units; its slider bar runs to
// where the value sits in its SoftLimit.
import { describe, expect, test } from "bun:test";
import { AngleDegrees, Dimensionless, FeathersNumberInput, LengthMeters, NumberInputRange, NumberInputValue, TimeSeconds, UiTheme, create_dark_theme, tokens } from "raea/feathers";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(<UiTheme theme={create_dark_theme()}>{el}</UiTheme>, { frame: 0, width: 320, height: 240, fps: 60, durationInFrames: 10 });
const show = (v: NumberInputValue) => Dimensionless.format(v, false);

describe("a number's text, as Rust's {} prints it", () => {
  test("the shortest digits that read back as the same f32 or f64, with no .0 and no exponent", () => {
    expect(show(NumberInputValue.F32(1))).toBe("1");
    expect(show(NumberInputValue.F32(0.1))).toBe("0.1");
    expect(show(NumberInputValue.F32(1 / 3))).toBe("0.33333334");
    expect(show(NumberInputValue.F64(1 / 3))).toBe("0.3333333333333333");
    expect(show(NumberInputValue.F32(1e-7))).toBe("0.0000001");
    expect(show(NumberInputValue.F64(1e21))).toBe("1000000000000000000000");
    expect(show(NumberInputValue.F32(-0))).toBe("-0");
    expect(show(NumberInputValue.F32(NaN))).toBe("NaN");
    expect(show(NumberInputValue.F64(-Infinity))).toBe("-inf");
    expect(show(NumberInputValue.I32(-7))).toBe("-7");
  });
  test("units scale the canonical value and append their suffix; degrees edit as d", () => {
    expect(AngleDegrees.format(NumberInputValue.F32(Math.PI), false)).toBe("180°");
    expect(AngleDegrees.format(NumberInputValue.F32(Math.PI), true)).toBe("180d");
    expect(LengthMeters.format(NumberInputValue.F32(2), false)).toBe("2m");
    expect(TimeSeconds.format(NumberInputValue.I32(3), false)).toBe("3s");
  });
  test("an integer outside its type is refused", () => {
    expect(() => NumberInputValue.I32(2 ** 31)).toThrow(/whole number/);
    expect(() => NumberInputValue.I32(1.5)).toThrow(/whole number/);
  });
});

describe("FeathersNumberInput", () => {
  test("its value held within its HardLimit, the bar to its place in its SoftLimit", () => {
    const out = html(<FeathersNumberInput value={NumberInputValue.F32(250)} hard_limit={NumberInputRange.F32(-100, 100)} soft_limit={NumberInputRange.F32(0, 400)} />);
    expect(out).toContain(">100<");
    expect(out).toMatch(/linear-gradient\(1.5707963267948966rad in srgb, rgba\(32, 110, 201, 1\) 0%, rgba\(32, 110, 201, 1\) 25%, rgba\(54, 55, 59, 1\) 25%/);
  });
  test("no SoftLimit, no bar; a label adds a 4 px border in the sigil's colour", () => {
    expect(html(<FeathersNumberInput value={NumberInputValue.F32(1)} />)).toMatch(/1\) 0%, rgba\(54, 55, 59, 1\) 0%/);
    expect(html(<FeathersNumberInput value={NumberInputValue.F32(1)} sigil_color={tokens.TEXT_INPUT_X_AXIS} label_text="X" />)).toMatch(/border-width:0px 0px 0px 4px;border-style:solid;border-color:rgba\(171, 64, 81, 1\)/);
  });
  test("a range of another type than the value is refused", () => {
    expect(() => html(<FeathersNumberInput value={NumberInputValue.F32(1)} soft_limit={NumberInputRange.I32(0, 10)} />)).toThrow(/NumberInputRange.F32/);
  });
});
