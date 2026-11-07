// docs/control.feature FC0a: bevy_ui's Node, Text and TextSpan on the page, with Bevy's defaults, values and blend
// (ui/styling/transparency_ui, ui/layout/fixed_node).
import { describe, expect, test } from "bun:test";
import { BorderRadius, FontVariationTag, ImageNode, ImagePlugin, Node, TextureAtlasLayout, Overflow, OverflowClipMargin, Text, Text2d, UiScale, TextSpan, UiRect, auto, percent, px, vw } from "raea/ui";
import { LinearRgba, Srgba } from "raea/color";
import { renderFrame } from "../src/test";
import { Transform } from "raea/transform";
import { Quat } from "raea/transform";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 30, durationInFrames: 30 });

describe("Node: bevy_ui's box with Bevy's defaults", () => {
  test("a root sits in its own implicit viewport (a full-size grid, its item at the start); a child doesn't", () => {
    const out = html(<Node width={px(10)}><Node height={px(5)} /></Node>);
    expect(out.match(/display:grid/g)?.length).toBe(1);
    expect(out).toContain("position:absolute;left:0;top:0;width:100%;height:100%;display:grid;align-items:start;justify-items:start");
    expect(out.match(/display:flex;box-sizing:border-box;position:relative;flex-shrink:1;min-width:auto;min-height:auto/g)?.length).toBe(2);
  });
  test("a FixedNode is a root wherever it is, fixed to the window", () => {
    const out = html(<Node><Node fixed width={px(100)} /></Node>);
    expect(out.match(/display:grid/g)?.length).toBe(2);
    expect(out).toContain("position:fixed;left:0;top:0");
  });
  test("its fields as CSS: Vals, enums (Default sends nothing), UiRects in CSS's order, radii, overflow", () => {
    const out = html(
      <Node width={percent(50)} min_height={vw(10)} max_width={auto} flex_direction="ColumnReverse" justify_content="SpaceBetween" align_items="Default"
        margin={UiRect.axes(px(1), px(2))} padding={UiRect.new(px(1), px(2), px(3), px(4))} border_radius={BorderRadius.top(px(6))} overflow={Overflow.clip_y()} flex_grow={2} />,
    );
    for (const s of ["width:50%", "min-height:10vw", "max-width:auto", "flex-direction:column-reverse", "justify-content:space-between", "margin:2px 1px 2px 1px", "padding:3px 2px 4px 1px", "border-radius:6px 6px 0px 0px", "overflow-x:visible;overflow-y:clip", "flex-grow:2"]) expect(out).toContain(s);
    expect(out).not.toContain("align-items:normal");
    expect(out).not.toContain("align-items:default");
  });
  test("a border is solid, transparent until a border_color colours it", () => {
    expect(html(<Node border={UiRect.all(px(2))} />)).toContain("border-width:2px 2px 2px 2px;border-style:solid;border-color:transparent");
    expect(html(<Node border={UiRect.all(px(2))} border_color={new Srgba(1, 0, 0)} />)).toContain("border-color:rgba(255, 0, 0, 1)");
  });
  test("a Val, an enum or an overflow that isn't Bevy's is refused", () => {
    expect(() => html(<Node width={"10" as never} />)).toThrow(/width is a Val/);
    expect(() => html(<Node justify_content={"Middle" as never} />)).toThrow(/justify_content is one of/);
    expect(() => html(<Node overflow={{ x: "Clip", y: "Auto" } as never} />)).toThrow(/overflow.y/);
    expect(() => px(Number.NaN)).toThrow(/finite/);
  });
});

describe("Bevy's blend: translucent colours over a known backdrop in linear light", () => {
  test("white at 0.2 over srgb 0.1 green is Bevy's (126, 164, 126), not the page's sRGB blend (72, 153, 72)", () => {
    const out = html(<Node background_color={new Srgba(0.1, 0.5, 0.1)}><Text color={new Srgba(1, 1, 1, 0.2)}>a</Text></Node>);
    expect(out).toContain("color:rgba(126, 164, 126, 1)");
  });
  test("a translucent background over an opaque one is the backdrop of what's on it", () => {
    const out = html(
      <Node background_color={new Srgba(0, 0, 0)}>
        <Node background_color={new Srgba(1, 1, 1, 0.5)}><Text color={new LinearRgba(0, 0, 0, 0.5)}>a</Text></Node>
      </Node>,
    );
    // linear 0.5 white is srgb 188; half of black over it, linear 0.25, is srgb 137
    expect(out).toContain("background-color:rgba(188, 188, 188, 1)");
    expect(out).toContain("color:rgba(137, 137, 137, 1)");
  });
  test("over nothing known (a root over the 3D) a translucent colour is sent as it is", () => {
    expect(html(<Node background_color={new Srgba(1, 0, 0, 0.5)} />)).toContain("background-color:rgba(255, 0, 0, 0.5)");
  });
});

describe("Text and TextSpan: bevy_text's defaults, and a span takes nothing from its Text", () => {
  test("Bevy's default font at 20 px, white, line height 1.2, unsmoothed, wrapped at words", () => {
    const out = html(<Text>hi</Text>);
    for (const s of ['font-family:&quot;Bevy Default Font&quot;', "font-size:20px", "color:#ffffff", "line-height:1.2", "-webkit-font-smoothing:antialiased", "white-space:pre-wrap", "data:font/ttf;base64,"]) expect(out).toContain(s);
  });
  test("its font file, size units (Rem is bevy_text's 20 px), justify and linebreak", () => {
    const out = html(<Text font="fonts/a.ttf" font_size={{ Rem: 2 }} justify="Center" linebreak="NoWrap" line_height={{ Px: 30 }}>hi</Text>);
    for (const s of ["font-size:40px", "text-align:center", "white-space:pre", "line-height:30px", 'url("fonts/a.ttf")']) expect(out).toContain(s);
    expect(html(<Text font_size={33}>x</Text>)).toContain("font-size:33px");
  });
  test("a TextSpan's style is its own: Bevy's defaults where it gives none, not its Text's", () => {
    const out = html(<Text font_size={33} color={new Srgba(1, 0, 0)}>a<TextSpan>b</TextSpan></Text>);
    expect(out).toContain("font-size:33px");
    expect(out.match(/font-size:20px/g)?.length).toBe(1);
  });
  test("a size, weight, line height, justify or linebreak that isn't Bevy's is refused", () => {
    expect(() => html(<Text font_size={0}>x</Text>)).toThrow(/above 0/);
    expect(() => html(<Text font_size={{ Em: 2 } as never}>x</Text>)).toThrow(/font_size is/);
    expect(() => html(<Text weight={1500}>x</Text>)).toThrow(/weight/);
    expect(() => html(<Text line_height={{ Em: 1 } as never}>x</Text>)).toThrow(/line_height/);
    expect(() => html(<Text justify={"Middle" as never}>x</Text>)).toThrow(/justify/);
    expect(() => html(<Text linebreak={"Never" as never}>x</Text>)).toThrow(/linebreak/);
  });
});

describe("ImageNode, Underline and Strikethrough", () => {
  test("an ImageNode is its picture on a Node, filling its content box; a root one sits in its viewport", () => {
    const out = html(<ImageNode image="branding/icon.png" width={px(256)} padding={UiRect.all(px(10))} flip_x />);
    expect(out).toContain('<img src="branding/icon.png"');
    for (const s of ["display:grid", "width:256px", "padding:10px 10px 10px 10px", "object-fit:fill", "transform:scaleX(-1)"]) expect(out).toContain(s);
    expect(html(<Node><ImageNode image="a.png" color={new Srgba(0.5, 0.5, 0.5)} /></Node>)).toContain("filter:brightness(0.5)");
  });
  test("an ImageNode with children, a coloured tint or no picture is refused", () => {
    expect(() => html(<ImageNode image="a.png"><Node /></ImageNode>)).toThrow(/no children/);
    expect(() => html(<ImageNode image="a.png" color={new Srgba(1, 0, 0)} />)).toThrow(/grey/);
    expect(() => html(<ImageNode image="" />)).toThrow(/picture file/);
  });
  test("Underline and Strikethrough are lines on the text; a font file's face spans every weight (a variable font's axis)", () => {
    // inside the text's own block (the inline root Blitz draws decorations from), not on the Node around it; as thick
    // as the font says; the block measured as bevy_text measures (whole pixels)
    expect(html(<Text underline strikethrough>a</Text>)).toContain('<span data-raea-bevy-text="" style="white-space:pre-wrap"><span style="text-decoration-line:underline line-through;text-decoration-thickness:from-font">a</span></span>');
    expect(html(<Text>a</Text>)).not.toContain("text-decoration");
    // a Text's lines are its own string's: its spans aren't struck (CSS would carry them to every span inside)
    const mixed = html(<Text strikethrough>a<TextSpan>b</TextSpan></Text>);
    expect(mixed).toMatch(/<span style="text-decoration-line:line-through;text-decoration-thickness:from-font">a<\/span>/);
    expect(mixed).toMatch(/<span style="[^"]*antialiased">b<\/span>/);
    expect(mixed.split("line-through").length - 1).toBe(1);
    // two colours of line are two spans; a span takes its own lines, colours and background
    expect(html(<TextSpan underline strikethrough underline_color={new Srgba(1, 1, 0)} strikethrough_color={new Srgba(1, 1, 1)} text_background_color={new Srgba(0, 0, 0)}>c</TextSpan>)).toContain(
      '<span style="background-color:rgba(0, 0, 0, 1);text-decoration-line:underline;text-decoration-thickness:from-font;text-decoration-color:rgba(255, 255, 0, 1)"><span style="text-decoration-line:line-through;text-decoration-thickness:from-font;text-decoration-color:rgba(255, 255, 255, 1)">c</span></span>',
    );
    expect(html(<Text font="f.ttf" weight={134}>a</Text>)).toContain('src: url("f.ttf"); font-weight: 1 1000;');
  });
  test("FontVariations set a variable font's axes by tag; a bad tag or value is refused", () => {
    expect(html(<Text font="f.ttf" font_variations={{ [FontVariationTag.WEIGHT]: 300, wdth: 80 }}>a</Text>)).toContain("font-variation-settings:&quot;wght&quot; 300, &quot;wdth&quot; 80");
    expect(html(<Text>a</Text>)).not.toContain("font-variation-settings");
    expect(() => html(<Text font_variations={{ wg: 300 }}>a</Text>)).toThrow(/four characters/);
    expect(() => html(<Text font_variations={{ wght: NaN }}>a</Text>)).toThrow(/is a number/);
  });
  test("a TextSpan's TextBackgroundColor is a box behind its run, in its own colour", () => {
    expect(html(<Text><TextSpan text_background_color={new Srgba(1, 0, 0)}>a</TextSpan></Text>)).toMatch(/<span style="[^"]*background-color:rgba\(255, 0, 0, 1\)[^"]*">a<\/span>/);
    expect(html(<Text><TextSpan>a</TextSpan></Text>)).not.toContain("background-color");
  });
  test("an Outline is a line outside the border box, its offset from it", () => {
    expect(html(<Node outline={{ width: px(8), offset: px(2), color: new Srgba(1, 0, 0) }} />)).toContain("outline:8px solid rgba(255, 0, 0, 1);outline-offset:2px");
  });
  test("TextureAtlasLayout.from_grid lays cells out as Bevy's does; an atlas ImageNode shows one cell, sized by it", () => {
    const l = TextureAtlasLayout.from_grid([24, 24], 7, 1);
    expect(l.size).toEqual([168, 24]);
    expect(l.textures[3]).toEqual({ x: 72, y: 0, w: 24, h: 24 });
    // padding between cells (not after the last) and an offset before the first, which isn't in the size
    const p = TextureAtlasLayout.from_grid([10, 10], 2, 2, [2, 3], [5, 5]);
    expect(p.size).toEqual([22, 23]);
    expect(p.textures[3]).toEqual({ x: 17, y: 18, w: 10, h: 10 });
    const out = html(<ImageNode image="s.png" texture_atlas={{ layout: l, index: 2 }} />);
    expect(out).toContain("width:24px;height:24px");
    expect(out).toContain("left:-200%;top:0%;width:700%;height:100%");
    expect(() => html(<ImageNode image="s.png" texture_atlas={{ layout: l, index: 7 }} />)).toThrow(/not one of its layout's 7 cells/);
  });
  test("ImagePlugin's default sampler: Nearest draws its pictures pixelated, Linear (the default) smooth", () => {
    expect(html(<ImagePlugin default_sampler="Nearest"><Node><ImageNode image="a.png" /></Node></ImagePlugin>)).toContain("image-rendering:pixelated");
    expect(html(<ImagePlugin><Node><ImageNode image="a.png" /></Node></ImagePlugin>)).not.toContain("image-rendering");
    expect(() => html(<ImagePlugin default_sampler={"Bilinear" as never} />)).toThrow(/Linear or Nearest/);
  });
});

describe("Text2d: a Text in Bevy's 2D world", () => {
  test("its block's centre is at its Transform (Anchor::CENTER), the frame's centre the origin, +Y up", () => {
    const out = html(<Text2d transform={Transform.fromXyz(-100, 50, 0)} font_size={30}>Repeat On</Text2d>);
    for (const s of ["left:60px", "top:70px", "transform:translate(-50%, -50%) rotate(0rad) scale(1, 1)", "font-size:30px", "data-raea-bevy-text", "Repeat On"]) expect(out).toContain(s);
    // its Text is placed by the Text2d, not made a root of its own that fills the frame
    expect(out.match(/width:100%;height:100%/g)?.length).toBe(1);
    const turned = html(<Text2d transform={Transform.fromXyz(0, 0, 0).withRotation(Quat.fromRotationZ(0.5)).withScale([2, 3, 1])}>x</Text2d>);
    expect(turned).toContain("rotate(-0.5rad) scale(2, 3)");
  });
});

describe("OverflowClipMargin: the box an overflow clip clips to", () => {
  test("its visual box and margin as overflow-clip-margin, Bevy's constructors, and a bad one refused", () => {
    expect(html(<Node overflow={Overflow.clip()} overflow_clip_margin={OverflowClipMargin.border_box().with_margin(25)} />)).toContain("overflow-clip-margin:border-box 25px");
    expect(html(<Node overflow={Overflow.clip()} overflow_clip_margin={OverflowClipMargin.content_box()} />)).toContain("overflow-clip-margin:content-box 0px");
    expect(OverflowClipMargin.DEFAULT).toEqual({ visual_box: "PaddingBox", margin: 0 });
    expect(html(<Node overflow={Overflow.clip()} />)).not.toContain("overflow-clip-margin");
    expect(() => html(<Node overflow_clip_margin={{ visual_box: "MarginBox" as never, margin: 0 }} />)).toThrow(/overflow_clip_margin is/);
    expect(() => html(<Node overflow_clip_margin={{ visual_box: "BorderBox", margin: -1 }} />)).toThrow(/px >= 0/);
  });
});

describe("A sliced ImageNode, a sampled picture and UiScale", () => {
  const slicer = { border: { left: 16, right: 8, top: 4, bottom: 2 }, sides_scale_mode: { Tile: { stretch_value: 1.5 } }, max_corner_scale: 2 } as const;
  test("NodeImageMode::Sliced is CSS border-image with fill, and bevy_ui's scale modes on data-raea-bevy-slice", () => {
    const out = html(<ImageNode image="p.png" image_mode={{ Sliced: slicer }} width={px(100)} height={px(50)}><Text>hi</Text></ImageNode>);
    for (const s of ['border-image-source:url(&quot;p.png&quot;)', "border-image-slice:4 8 2 16 fill", "border-image-width:8px 16px 4px 32px", 'data-raea-bevy-slice="2 t1.5 s"', "hi"]) expect(out).toContain(s);
    expect(() => html(<ImageNode image="p.png" image_mode={{ Sliced: { ...slicer, center_scale_mode: { Tile: { stretch_value: 0 } } } }} />)).toThrow(/center_scale_mode is/);
    expect(() => html(<ImageNode image="p.png" image_mode={{ Sliced: { border: { left: -1, right: 0, top: 0, bottom: 0 } } }} />)).toThrow(/px >= 0/);
  });
  test("a flipped one draws its slices on a mirrored layer behind its children", () => {
    const out = html(<ImageNode image="p.png" flip_x image_mode={{ Sliced: slicer }}><Text>hi</Text></ImageNode>);
    for (const s of ["z-index:-1", "transform:scaleX(-1)", "position:relative", "z-index:0"]) expect(out).toContain(s);
    expect(out.indexOf("scaleX(-1)")).toBeLessThan(out.indexOf("hi"));
  });
  test("an image with its own sampler is drawn by its filter, and a wrap is refused", () => {
    expect(html(<ImageNode image={{ path: "p.png", sampler: { filter: "nearest" } }} />)).toContain("image-rendering:pixelated");
    expect(html(<ImageNode image={{ path: "p.png", sampler: {} }} />)).not.toContain("image-rendering");
    expect(() => html(<ImageNode image={{ path: "p.png", sampler: { wrap: "repeat" } }} />)).toThrow(/isn't wrapped/);
  });
  test("UiScale lays its roots out in a window 1/value of the frame and scales it up from the top left", () => {
    const out = html(<UiScale value={2}><Node width={px(16)} /></UiScale>);
    for (const s of ["width:50%", "height:50%", "transform:scale(2)", "transform-origin:0 0"]) expect(out).toContain(s);
    expect(() => html(<UiScale value={0} />)).toThrow(/above 0/);
  });
});
