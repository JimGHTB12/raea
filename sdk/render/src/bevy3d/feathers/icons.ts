// bevy_feathers' icons (constants.rs icons; docs/control.feature FC0a): its own 24 x 24 PNGs (MIT OR Apache-2.0, as
// bevy_feathers 0.20.0 ships them in src/assets/icons), inline as data URLs so a render needs no files. Each is a white
// shape on transparency, which icon() tints with the text colour around it.

/** a feathers icon: its picture and size in px */
export interface FeathersIcon { image: string; width: number; height: number }

/** bevy_feathers' icons. @bevy bevy_feathers::constants::icons */
export const icons = {
  CHEVRON_DOWN: { image: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAAAXNSR0IArs4c6QAAAItJREFUSEvtklEKgCAQRGdOVt2sTladbEMxMBF3CqQf/RFpd168XaLzYed8DIBreCj6psjM1tBJMt7eMbMdwFmrr84gNcwADpJLC5DVbjIgBCoQpaa5Ra0AJTxqFv0+dKnhEqDUlX5Imo8MKCDh6Q7/NuMqyhWm9Z28zcp7XgG8edW+D4BrbSj6X9EFizdAGf6qyhEAAAAASUVORK5CYII=", width: 24, height: 24 } as FeathersIcon,
  CHEVRON_RIGHT: { image: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAAAXNSR0IArs4c6QAAAIhJREFUSEvt1EEOgCAMBMDuy9SXqS/Tn1UheDTdrnKDxHghO7YVYJ0XOufbAMIOhy1y962kAKjv7GKAw8zm+zkBLL8DJdDdZSSs4PliFaEBtZIUoCBpIIt8Bfbo900D2WGngGx4PaDswVHCaUANp4B2F63drooGTMo9RFXAzuhtHz1kFRpA2LkLh4JEGZFKoL4AAAAASUVORK5CYII=", width: 24, height: 24 } as FeathersIcon,
  X: { image: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAAAXNSR0IArs4c6QAAALlJREFUSEvtVMERg0AIXJpIHfloC/lYrp+0kHxSh03g4GDGOeXgRvl5771dWBYIyY+S+XELuA4fWsTMH/05ENFUY2HmB4BRMETUl9iaQAfgC8AU2ZAv2BaBtSpTpCS3CjFTVCOIki+2Bf39d6J48dy10BUQQFHtTwWe3nzWwkOLpiJvAEIsT4ReXsJCHWy6yBFItegoLZcNOTWmkZxHMOaQ9Ra5OT9zKnKPnXuDGwChRWvg20FvAde9GfckghnaevL3AAAAAElFTkSuQmCC", width: 24, height: 24 } as FeathersIcon,
};
