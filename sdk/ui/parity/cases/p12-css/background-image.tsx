// background-image url(): data: URL pictures (PNG base64, SVG percent-encoded) sized, placed and repeated, scaled up
// pixelated as a C64 game's sprites are, layered under a gradient, from the longhands and the shorthand.
// proves: css.properties.background-image, css.properties.background-size, css.properties.background-size.cover, css.properties.background-size.contain, css.properties.background-size.auto, css.properties.background-position, css.properties.background-position.right, css.properties.background-position.bottom, css.properties.background-position.center, css.properties.background-repeat, css.properties.background-repeat.no-repeat, css.properties.background-repeat.repeat, css.properties.background-repeat.repeat-x, css.properties.background-repeat.repeat-y, css.properties.image-rendering, css.properties.image-rendering.pixelated, css.properties.background
export const steps = [];

// a 4 x 4 checker (red and blue, one pixel clear) and an 8 x 8 of four quarters (yellow, clear, blue, green)
const CHECK = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAQAAAAECAYAAACp8Z5+AAAAHklEQVR4nGN4KOvxXzX59X8YzQAmgAAmyIBVBbIgAI/II3ROl1rOAAAAAElFTkSuQmCC";
const QUAD = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAAIUlEQVR4nGP4dUb0PzJmQAd0UKCa/Po/MlY6GoeC6aAAABvgeQHBIU/gAAAAAElFTkSuQmCC";
const DOT = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect x='4' y='4' width='32' height='32' fill='%2314b8a6'/%3E%3C/svg%3E";

const px = { imageRendering: "pixelated" } as const;

export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-wrap content-start gap-4 bg-slate-800 p-4">
      <div data-testid="scaled" className="h-16 w-16" style={{ ...px, backgroundImage: `url("${CHECK}")`, backgroundSize: "64px 64px", backgroundRepeat: "no-repeat" }} />
      <div data-testid="tiled" className="h-16 w-32" style={{ ...px, backgroundImage: `url("${CHECK}")`, backgroundSize: "16px", backgroundPosition: "6px 3px" }} />
      <div data-testid="repeat-x" className="h-16 w-40 bg-slate-600" style={{ ...px, backgroundImage: `url(${QUAD})`, backgroundSize: "24px 24px", backgroundRepeat: "repeat-x", backgroundPosition: "center" }} />
      <div data-testid="repeat-y" className="h-24 w-16 bg-slate-600" style={{ ...px, backgroundImage: `url(${QUAD})`, backgroundSize: "16px 16px", backgroundRepeat: "repeat-y", backgroundPosition: "right top" }} />
      <div data-testid="edges" className="h-16 w-32 bg-slate-950" style={{ ...px, backgroundImage: `url(${QUAD})`, backgroundSize: "32px", backgroundRepeat: "no-repeat", backgroundPosition: "right 8px bottom 4px" }} />
      <div data-testid="contain" className="h-16 w-32 bg-slate-950" style={{ ...px, backgroundImage: `url(${QUAD})`, backgroundSize: "contain", backgroundRepeat: "no-repeat", backgroundPosition: "center" }} />
      <div data-testid="cover" className="h-24 w-12" style={{ ...px, backgroundImage: `url(${QUAD})`, backgroundSize: "cover", backgroundRepeat: "no-repeat", backgroundPosition: "left top" }} />
      <div data-testid="natural" className="h-12 w-24 bg-slate-950" style={{ ...px, backgroundImage: `url(${CHECK})`, backgroundRepeat: "no-repeat", backgroundPosition: "50% 50%" }} />
      <div data-testid="bordered" className="h-16 w-24" style={{ ...px, border: "6px solid rgb(148, 163, 184)", backgroundImage: `url(${CHECK})`, backgroundSize: "12px" }} />
      <div data-testid="layers" className="h-16 w-32" style={{ ...px, backgroundImage: `linear-gradient(90deg, rgba(15, 23, 42, 0.8), rgba(15, 23, 42, 0)), url(${CHECK})`, backgroundSize: "auto, 16px 16px" }} />
      <div data-testid="shorthand" className="h-16 w-32" style={{ ...px, background: `url(${QUAD}) center / 32px 32px no-repeat, rgb(76, 29, 149)` }} />
      <div data-testid="svg" className="h-16 w-16 bg-white" style={{ backgroundImage: `url("${DOT}")`, backgroundRepeat: "no-repeat", backgroundPosition: "center" }} />
    </div>
  );
}
