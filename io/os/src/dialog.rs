//! dialog.message: the system's message dialog (rfd), answered with the index of the button pressed; headless, the
//! answer a dialog step queued.

#[cfg(not(target_os = "ios"))]
use rfd::{AsyncMessageDialog, MessageButtons, MessageDialogResult, MessageLevel};
use serde_json::{Value, json};

use crate::{Done, Scripted, headless, scripted};

pub(crate) fn message(args: &Value, done: Done) {
    let labels: Vec<String> = args["buttons"].as_array().map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default();
    if labels.len() > 3 {
        return done(Err(format!("dialog.message: {} buttons, at most 3", labels.len())));
    }
    if headless() {
        return done(match scripted(|s| if let Scripted::Dialog(i) = s { Some(*i) } else { None }) {
            Some(i) if i < labels.len().max(1) => Ok(json!(i)),
            Some(i) => Err(format!("dialog.message: the dialog step pressed button {i} of {}", labels.len().max(1))),
            None => Err("dialog.message: headless, and no dialog step answers it".into()),
        });
    }
    #[cfg(not(target_os = "ios"))]
    system(args, labels, done);
}

#[cfg(not(target_os = "ios"))]
fn system(args: &Value, labels: Vec<String>, done: Done) {
    let level = match args["kind"].as_str() {
        Some("warning") => MessageLevel::Warning,
        Some("error") => MessageLevel::Error,
        _ => MessageLevel::Info,
    };
    let buttons = match labels.as_slice() {
        [] => MessageButtons::Ok,
        [a] => MessageButtons::OkCustom(a.clone()),
        [a, b] => MessageButtons::OkCancelCustom(a.clone(), b.clone()),
        [a, b, c, ..] => MessageButtons::YesNoCancelCustom(a.clone(), b.clone(), c.clone()),
    };
    let d = AsyncMessageDialog::new()
        .set_title(args["title"].as_str().unwrap_or(""))
        .set_description(args["text"].as_str().unwrap_or(""))
        .set_level(level)
        .set_buttons(buttons);
    std::thread::spawn(move || {
        let r = pollster::block_on(d.show());
        let i = match r {
            MessageDialogResult::Custom(s) => labels.iter().position(|l| *l == s).unwrap_or(0),
            MessageDialogResult::Cancel | MessageDialogResult::No => 1,
            MessageDialogResult::Ok | MessageDialogResult::Yes => 0,
        };
        done(Ok(json!(i)));
    });
}
