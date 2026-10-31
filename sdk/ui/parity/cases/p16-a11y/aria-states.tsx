// ARIA states and properties on custom widgets: aria-pressed true and false, aria-expanded, aria-checked mixed and a
// switch, aria-selected, aria-disabled, aria-haspopup, aria-required, aria-readonly and aria-invalid on a textbox,
// aria-valuemin/max/now on a slider and a progressbar.
// proves: aria.state.aria-pressed, aria.state.aria-expanded, aria.state.aria-checked, aria.state.aria-selected, aria.state.aria-disabled, aria.state.aria-invalid, aria.property.aria-haspopup, aria.property.aria-required, aria.property.aria-readonly, aria.property.aria-valuemin, aria.property.aria-valuemax, aria.property.aria-valuenow, aria.role.switch, aria.role.progressbar, aria.role.slider
export const steps = [];
const f = "block h-6 w-32";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="grid grid-cols-3 gap-2 opacity-0">
        <button data-testid="pressed" aria-pressed="true" className={f}>Bold</button>
        <button data-testid="unpressed" aria-pressed="false" className={f}>Italic</button>
        <button data-testid="expanded" aria-expanded="true" className={f}>Less</button>
        <button data-testid="collapsed" aria-expanded="false" aria-haspopup="menu" className={f}>More</button>
        <div data-testid="mixed" role="checkbox" aria-checked="mixed" tabIndex={0} className={f}>All</div>
        <div data-testid="switch" role="switch" aria-checked="true" tabIndex={0} className={f}>Wifi</div>
        <div data-testid="option" role="option" aria-selected="true" className={f}>Picked</div>
        <button data-testid="aria-disabled" aria-disabled="true" className={f}>Nope</button>
        <div data-testid="textbox" role="textbox" aria-label="Code" aria-required="true" aria-invalid="true" tabIndex={0} className={f} />
        <div data-testid="ro" role="textbox" aria-label="Shown" aria-readonly="true" tabIndex={0} className={f} />
        <div data-testid="slider" role="slider" aria-label="Level" aria-valuemin={0} aria-valuemax={100} aria-valuenow={40} tabIndex={0} className={f} />
        <div data-testid="progress" role="progressbar" aria-label="Upload" aria-valuenow={20} className={f} />
      </div>
    </div>
  );
}
