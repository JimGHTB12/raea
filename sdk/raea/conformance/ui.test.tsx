// docs/control.feature FC0a: bevy_ui's Node, Text and TextSpan on the page, with Bevy's defaults, values and blend
// (ui/styling/transparency_ui, ui/layout/fixed_node).
import { describe, expect, test } from "bun:test";
import { AngularColorStop, FontFeatureTag, FontWidth, GhostNode, Viewport, UiTargetCamera, UiRect as GradRect, FontSource, GridPlacement, GridTrack, RepeatedGridTrack, ColorStop, ConicGradient, LinearGradient, RadialGradient, UiPosition, BorderRadius, CornerRadius, FontVariationTag, ImageNode, InlineImage, ImagePlugin, Node, TextureAtlasLayout, Overflow, OverflowClipMargin, Text, TextShadow, Text2d, UiScale, TextSpan, UiRect, auto, percent, px, vw } from "raea/ui";
import { LinearRgba, Srgba } from "raea/color";
import { Canvas3D } from "raea/3d";
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
  test("an ImageNode is its picture on a Node, contained in its content box (Auto); a root one sits in its viewport", () => {
    const out = html(<ImageNode image="branding/icon.png" width={px(256)} padding={UiRect.all(px(10))} flip_x />);
    expect(out).toContain('<img src="branding/icon.png"');
    for (const s of ["display:grid", "width:256px", "padding:10px 10px 10px 10px", "object-fit:contain", "transform:scaleX(-1)"]) expect(out).toContain(s);
    // Stretch: no <img> (no picture size for layout), the picture stretched over the content box
    const st = html(<Node><ImageNode image="a.png" image_mode="Stretch" width={percent(10)} /></Node>);
    expect(st).not.toContain("<img");
    for (const s of ["background-image:url(&quot;a.png&quot;)", "background-size:100% 100%", "background-origin:content-box"]) expect(st).toContain(s);
    expect(() => html(<ImageNode image="a.png" image_mode={"Fit" as "Auto"} />)).toThrow(/image_mode is "Auto", "Stretch" or/);
    expect(html(<Node><ImageNode image="a.png" color={new Srgba(0.5, 0.5, 0.5)} /></Node>)).toContain("filter:brightness(0.5)");
  });
  test("an ImageNode with children, a coloured tint or no picture is refused", () => {
    expect(() => html(<ImageNode image="a.png"><Node /></ImageNode>)).toThrow(/no children/);
    expect(() => html(<ImageNode image="a.png" color={new Srgba(1, 0, 0)} />)).toThrow(/grey/);
    expect(() => html(<ImageNode image="" />)).toThrow(/picture file/);
  });
  test("Underline and Strikethrough are lines on the text; a font file's face spans every weight (a variable font's axis)", () => {
    // on its own strings inside the text's block (the Text's box, the inline root Blitz draws decorations from); as
    // thick as the font says; the block measured as bevy_text measures (whole pixels)
    expect(html(<Text underline strikethrough>a</Text>)).toContain('white-space:pre-wrap" data-raea-bevy-text=""><span style="text-decoration-line:underline line-through;text-decoration-thickness:from-font">a</span></div>');
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
  test("FontFeatures turn OpenType features on by tag, on a Text or a TextSpan; a bad tag or value is refused", () => {
    expect(html(<Text font_features={{ [FontFeatureTag.SMALL_CAPS]: 1, [FontFeatureTag.FRACTIONS]: 1 }}>a</Text>)).toContain("font-feature-settings:&quot;smcp&quot; 1, &quot;frac&quot; 1");
    expect(html(<Text>a<TextSpan font_features={{ [FontFeatureTag.OLDSTYLE_FIGURES]: 1 }}>1</TextSpan></Text>)).toMatch(/<span style="[^"]*font-feature-settings:&quot;onum&quot; 1[^"]*">1<\/span>/);
    expect(html(<Text>a</Text>)).not.toContain("font-feature-settings");
    expect(() => html(<Text font_features={{ sc: 1 }}>a</Text>)).toThrow(/four characters/);
    expect(() => html(<Text font_features={{ smcp: 0.5 }}>a</Text>)).toThrow(/whole number/);
  });
  test("TextFont's width (font_width, FontWidth's shares) is CSS's font-stretch, on a Text or a TextSpan", () => {
    expect(html(<Text font_width={FontWidth.CONDENSED}>a</Text>)).toContain("font-stretch:75%");
    expect(html(<Text>a<TextSpan font_width={FontWidth.ULTRA_EXPANDED}>b</TextSpan></Text>)).toMatch(/<span style="[^"]*font-stretch:200%[^"]*">b<\/span>/);
    expect(html(<Text>a</Text>)).not.toContain("font-stretch");
    expect(() => html(<Text font_width={0}>a</Text>)).toThrow(/font_width is above 0/);
  });
  test("a TextSpan of size 0 or less is left out, as bevy_text leaves it, and its Text's other runs drawn", () => {
    const out = html(<Text>a<TextSpan font_size={{ Px: 0 }}>zero</TextSpan><TextSpan font_size={-42}>negative</TextSpan><TextSpan>b</TextSpan></Text>);
    expect(out).not.toContain("zero");
    expect(out).not.toContain("negative");
    expect(out).toMatch(/>b<\/span>/);
    expect(() => html(<Text><TextSpan font_size={NaN}>x</TextSpan></Text>)).toThrow(/font_size/);
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
  test("a Text's box is its text's block, a measured leaf (a block container), so its lines are justified within it", () => {
    const out = html(<Text justify="Right" width={percent(100)}>hi</Text>);
    expect(out).toMatch(/<div style="display:flow-root;[^"]*width:100%[^"]*text-align:right;white-space:pre-wrap" data-raea-bevy-text="">hi<\/div>/);
    expect(html(<Text display="None">hi</Text>)).toContain("display:none");
  });
  test("InlineImage is a picture in the line, sized by width or height and its aspect, tinted by multiplying it over the colour, its alpha the mask", () => {
    const out = html(<Text>a<InlineImage image="p.png" color={new Srgba(1, 0, 0, 0.5)} height={30} /></Text>);
    for (const s of ['background-image:url(&quot;p.png&quot;), linear-gradient(rgba(255, 0, 0, 1), rgba(255, 0, 0, 1))', "background-blend-mode:multiply", 'mask-image:url(&quot;p.png&quot;)', "opacity:0.5", '<img src="p.png" style="display:block;height:30px;opacity:0"/>']) expect(out).toContain(s);
    expect(html(<Text><InlineImage image="p.png" width={30} /></Text>)).toContain('<span style="display:inline-block"><img src="p.png" style="display:block;width:30px;opacity:1"/></span>');
    expect(() => html(<Text><InlineImage image="p.png" width={0} /></Text>)).toThrow(/width is px above 0/);
  });
  test("corners resolve as Bevy's do with the node's px size: circular ones clamped to half its shorter side, elliptical ones per axis", () => {
    const out = html(<Node width={px(50)} height={px(40)} border_radius={BorderRadius.px(3.4e38, 0, 10, 0)} />);
    expect(out).toContain("border-radius:20px 0px 10px 0px");
    const ell = html(<Node width={px(50)} height={px(40)} border_radius={BorderRadius.all(CornerRadius.new(percent(50), percent(20)))} />);
    expect(ell).toContain("border-radius:25px 25px 25px 25px / 8px 8px 8px 8px");
    expect(html(<Node border_radius={BorderRadius.all(px(8))} />)).toContain("border-radius:8px 8px 8px 8px");
  });
  test("every node is a stacking context, so a ZIndex ranks it among its siblings above their parent; a GlobalZIndex is marked for the root's", () => {
    expect(html(<Node />)).toContain("z-index:0");
    expect(html(<Node z_index={-1} />)).toContain("z-index:-1");
    const g = html(<Node><Node global_z_index={-1} z_index={3} /></Node>);
    expect(g).toMatch(/z-index:-1" data-raea-bevy-global-z=""/);
    expect(html(<Node><Node z_index={2} /></Node>)).not.toContain("data-raea-bevy-global-z");
  });
  test("a layered Canvas3D cleared to a colour lies under the whole page, as a Bevy camera's output under all its UI", () => {
    expect(html(<Canvas3D layered clearColor="rgb(0, 0, 0)" />)).toContain("z-index:-2147483648");
    expect(html(<Canvas3D layered />)).not.toContain("z-index");
  });
  test("BackgroundGradient is CSS gradients in Oklab, the first at the bottom, Bevy's shapes and positions as CSS's", () => {
    const red = new Srgba(1, 0, 0), clear = new Srgba(1, 0, 0, 0);
    const out = html(<Node background_color={new Srgba(0, 0, 0)} background_gradient={[
      LinearGradient.to_right([ColorStop.auto(red), ColorStop.new(clear, percent(80))]),
      RadialGradient.new(UiPosition.TOP.at_x(percent(5)), "ClosestCorner", [ColorStop.auto(red), ColorStop.auto(clear)]),
      ConicGradient.new(UiPosition.BOTTOM_RIGHT.at(px(10), px(20)), [AngularColorStop.auto(red), AngularColorStop.new(clear, Math.PI)], 0.5),
    ]} />);
    expect(out).toContain("background-image:conic-gradient(from 0.5rad at calc(100% - 10px) calc(100% - 20px) in oklab, rgba(255, 0, 0, 1), rgba(255, 0, 0, 0) 3.141592653589793rad), radial-gradient(ellipse closest-side at calc(50% + 5%) calc(0% + 0px) in oklab, rgba(255, 0, 0, 1), rgba(255, 0, 0, 0)), linear-gradient(1.5707963267948966rad in oklab, rgba(255, 0, 0, 1), rgba(255, 0, 0, 0) 80%)");
    expect(out).toContain("background-origin:border-box");
    // a page with bevy_ui in it blends in linear light (raea-page's paint; vendor/vello_shaders fine.wgsl)
    expect(out).toContain('data-raea-blend="linear"');
    const hinted = html(<Node background_gradient={[LinearGradient.new(0, [{ color: red, point: px(0), hint: 0.25 }, ColorStop.new(clear, px(100))], "Srgba")]} />);
    expect(hinted).toContain("linear-gradient(0rad in srgb, rgba(255, 0, 0, 1) 0px, calc(0px + (100px - 0px) * 0.25), rgba(255, 0, 0, 0) 100px)");
    expect(() => html(<Node background_gradient={[RadialGradient.new(UiPosition.CENTER, { Circle: percent(10) }, [ColorStop.auto(red)])]} />)).toThrow(/Circle's radius in percent/);
  });
  test("BorderGradient is the bottom layer over the border box, the node's gradients over it clipped to the padding box, none without a border", () => {
    const g = [LinearGradient.to_right([ColorStop.auto(new Srgba(1, 1, 0)), ColorStop.auto(new Srgba(1, 0.5, 0))])];
    const own = [LinearGradient.to_top([ColorStop.auto(new Srgba(1, 0, 0)), ColorStop.auto(new Srgba(0, 0, 1))])];
    const out = html(<Node border={GradRect.all(px(4))} background_gradient={own} border_gradient={g} />);
    expect(out).toContain("background-image:linear-gradient(0rad in oklab, rgba(255, 0, 0, 1), rgba(0, 0, 255, 1)), linear-gradient(1.5707963267948966rad in oklab, rgba(255, 255, 0, 1), rgba(255, 128, 0, 1))");
    expect(out).toContain("background-clip:padding-box, border-box");
    expect(html(<Node border={GradRect.all(px(0))} background_gradient={own} border_gradient={g} />)).not.toContain("background-clip");
    expect(() => html(<Node border={GradRect.all(px(4))} background_gradient={[LinearGradient.to_top([ColorStop.auto(new Srgba(1, 0, 0, 0.5))])]} border_gradient={g} />)).toThrow(/translucent background_gradient/);
  });
  test("a grid's tracks and placements are CSS's: Bevy's fr is CSS's, its flex minmax(0, fr), repeats and spans kept", () => {
    const out = html(<Node display="Grid" grid_template_columns={[RepeatedGridTrack.fr(3, 1), GridTrack.px(40), RepeatedGridTrack.flex("AutoFill", 2)]} grid_template_rows={[GridTrack.minmax({ Px: 10 }, "MaxContent"), GridTrack.fit_content_px(50), GridTrack.rem(2)]} grid_auto_flow="ColumnDense" />);
    expect(out).toContain("grid-template-columns:repeat(3, 1fr) 40px repeat(auto-fill, minmax(0px, 2fr))");
    expect(out).toContain("grid-template-rows:minmax(10px, max-content) fit-content(50px) 40px");
    expect(out).toContain("grid-auto-flow:column dense");
    expect(html(<Node grid_column={GridPlacement.span(3)} grid_row={GridPlacement.end_span(-1, 2)} />)).toMatch(/grid-column:span 3;.*grid-row:span 2 \/ -1|grid-row:span 2 \/ -1;.*grid-column:span 3/);
    expect(html(<Node grid_column={GridPlacement.start(2)} />)).toContain("grid-column:2 / auto");
    expect(() => html(<Node grid_column={{ start: 1, end: 3, span: 2 }} />)).toThrow(/at most two/);
  });
  test("a Text's FontSource: a file as its own face, a family by name, a generic family as CSS's", () => {
    expect(html(<Text font={FontSource.fantasy()}>a</Text>)).toContain("font-family:&quot;raea-bevy fantasy&quot;, fantasy");
    expect(html(<Text font={FontSource.family("Times")}>a</Text>)).toContain("font-family:&quot;Times&quot;");
    expect(html(<Text font={FontSource.ui_rounded()}>a</Text>)).toContain("font-family:&quot;raea-bevy ui-rounded&quot;, ui-rounded");
    expect(() => html(<Text font={{ Generic: "Gothic" } as never}>a</Text>)).toThrow(/FontSource/);
  });
  test("a GhostNode has no box: inside a Node its children are the Node's (display: contents), at the top each is a root", () => {
    const inside = html(<Node><GhostNode><Node width={px(7)} /></GhostNode></Node>);
    expect(inside).toMatch(/<div style=\"display:contents\"><div style=\"display:flex[^\"]*width:7px/);
    const top = html(<GhostNode><Node width={px(7)} /><Node width={px(8)} /></GhostNode>);
    expect(top.match(/data-raea-blend=\"linear\"/g)?.length).toBe(2);
    expect(top).not.toContain("display:contents");
  });
  test("Visibility: Hidden is laid out but not drawn, Visible draws under a hidden parent, Inherited is the parent's", () => {
    expect(html(<Node visibility="Hidden" />)).toContain("visibility:hidden");
    expect(html(<Node visibility="Visible" />)).toContain("visibility:visible");
    expect(html(<Node visibility="Inherited" />)).not.toContain("visibility");
    expect(() => html(<Node visibility={"Gone" as never} />)).toThrow(/visibility/);
  });
  test("Viewport is the region of the window the UI's roots lay out in, clipped to it", () => {
    const out = html(<Viewport physical_position={[200, 100]} physical_size={[600, 500]}><Node /></Viewport>);
    expect(out).toContain("position:absolute;left:200px;top:100px;width:600px;height:500px;overflow:hidden");
    expect(() => html(<Viewport physical_position={[0, 0]} physical_size={[0, 5]} />)).toThrow(/physical_size/);
  });
  test("UiTargetCamera is a layer over the window stacked by its camera's order, its roots roots", () => {
    const out = html(<UiTargetCamera order={3}><Node width={px(9)} /></UiTargetCamera>);
    expect(out).toContain("z-index:3;isolation:isolate");
    expect(out).toContain('data-raea-blend="linear"');
    expect(() => html(<UiTargetCamera order={1.5} />)).toThrow(/whole number/);
  });
  test("BorderColor takes a colour per side", () => {
    const out = html(<Node border={UiRect.all(px(2))} border_color={{ top: new Srgba(1, 0, 0), right: new Srgba(0, 0, 1), bottom: new Srgba(1, 1, 0), left: new Srgba(0, 1, 0) }} />);
    expect(out).toContain("border-color:rgba(255, 0, 0, 1) rgba(0, 0, 255, 1) rgba(255, 255, 0, 1) rgba(0, 255, 0, 1)");
  });
  test("LetterSpacing is CSS letter-spacing, Rem of bevy_text's 20 px", () => {
    expect(html(<Text letter_spacing={{ Px: 3 }}>hi</Text>)).toContain("letter-spacing:3px");
    expect(html(<Text>a<TextSpan letter_spacing={{ Rem: 0.5 }}>b</TextSpan></Text>)).toContain("letter-spacing:10px");
    expect(() => html(<Text letter_spacing={{ Em: 1 } as never}>hi</Text>)).toThrow(/letter_spacing is/);
  });
  test("TextShadow is an unblurred text-shadow, its colour blended over the backdrop as Bevy blends in linear light", () => {
    expect(html(<Text shadow={TextShadow.default()}>hi</Text>)).toContain("text-shadow:4px 4px 0 rgba(0, 0, 0, 0.75)");
    expect(html(<Node background_color={new Srgba(0.15, 0.15, 0.15)}><Text shadow={{ offset: [2, -1], color: new LinearRgba(0, 0, 0, 0.75) }}>hi</Text></Node>)).toContain("text-shadow:2px -1px 0 rgba(15, 15, 15, 1)");
    expect(() => html(<Text shadow={{ offset: [NaN, 0], color: new Srgba(0, 0, 0) } as never}>hi</Text>)).toThrow(/shadow is/);
  });
  test("a sliced atlas cell puts its rect in its layout on the marker, which Blitz slices", () => {
    const layout = TextureAtlasLayout.from_grid([50, 50], 6, 6, [2, 2]);
    const out = html(<ImageNode image="p.png" texture_atlas={{ layout, index: 13 }} image_mode={{ Sliced: slicer }} />);
    expect(out).toContain('data-raea-bevy-slice="2 t1.5 s cell 52 104 50 50 of 310 310"');
    expect(() => html(<ImageNode image="p.png" texture_atlas={{ layout, index: 36 }} image_mode={{ Sliced: slicer }} />)).toThrow(/not one of its layout's 36 cells/);
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
  test("UiScale zooms its UI: laid out at value times every length, in the frame's window", () => {
    const out = html(<UiScale value={2}><Node width={px(16)} /></UiScale>);
    for (const s of ["width:100%", "height:100%", "zoom:2"]) expect(out).toContain(s);
    expect(out).not.toContain("transform:scale");
    expect(() => html(<UiScale value={0} />)).toThrow(/above 0/);
  });
});
