//! rune-kit's components in place of @raea/rune's stand-ins (R6a, sdk/rune/src/kit.tsx): an element whose
//! `data-rune-kit` names one is drawn as that component, built with the neutral kit (its dark one when the component
//! says `dark`; RollingText takes the CSS colour and font it inherits). The stand-in's own events still go back over
//! the wire (view.rs attaches them as for any element), so a Switch's press reaches React's onClick and the new
//! `checked` comes back in the next batch.

use rune::El;
use rune_kit::{Badge, BadgeVariant, Kit, Progress, RollingText, Spinner, Switch};
use serde_json::Value;

use crate::report::Report;

/// The component `json` describes for element `id`, or None (reported) when the host doesn't know it.
pub fn build(id: u64, json: &str, color: rune::Color, font: rune::TextStyle, path: &str, report: &mut Report) -> Option<El> {
    let v: Value = serde_json::from_str(json).ok()?;
    let k = Kit::neutral(v.get("dark").and_then(Value::as_bool).unwrap_or(false));
    let s = |f: &str| v.get(f).and_then(Value::as_str).unwrap_or("").to_string();
    Some(match v.get("kit").and_then(Value::as_str).unwrap_or("") {
        "badge" => {
            let variant = match s("variant").as_str() {
                "accent" => BadgeVariant::Accent,
                "subtle" => BadgeVariant::Subtle,
                _ => BadgeVariant::Neutral,
            };
            Badge::new(s("label")).variant(variant).build(&k)
        }
        "spinner" => Spinner::new().build(&k),
        "progress" => Progress::new(v.get("value").and_then(Value::as_f64).map(|x| x as f32)).build(&k),
        "rolling" => {
            let r = RollingText::new(format!("rolling-{id}"), s("text")).color(color).font(font);
            let r = match v.get("motion").and_then(crate::runeprops::spring) {
                Some(sp) => r.spring(sp),
                None => r,
            };
            r.build(&k)
        }
        "switch" => {
            let sw = Switch::new(v.get("checked").and_then(Value::as_bool).unwrap_or(false));
            let label = s("label");
            let sw = if label.is_empty() { sw } else { sw.label(label) };
            sw.build(&k)
        }
        other => {
            report.note(path, "data-rune-kit", other, "a rune-kit component the host doesn't know");
            return None;
        }
    })
}
