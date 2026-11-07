//! A scripted session (R4b): the steps js/apps/steps.ts names (a press, text typed into a field, a wheel over an
//! element, each by data-testid), plus a key at the focus and frames to wait, played through Rune's input. T3's set adds
//! a hover (the pointer left over an element), the system's appearance and the window's width (docs/targets.feature T3a).

use serde::Deserialize;

use crate::js::App;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Step {
    /// a press and release in the middle of the element
    Press { press: String },
    /// the field pressed, the caret to its end, and the text typed a character at a time
    Type { r#type: String, into: String },
    /// a wheel over the element's middle, `dy` points (positive scrolls the content up)
    Wheel { wheel: String, dy: f32 },
    /// a key at the focus, by DOM's name ("Enter", "Escape", "ArrowDown", "a")
    Key { key: String },
    /// frames to run with no input
    Frames { frames: u32 },
    /// the files the app's next fs.pick picks (T2d: a headless check opens no dialog)
    Pick { pick: Vec<String> },
    /// the file the app's next fs.save saves to
    Save { save: String },
    /// the button pressed in the app's next dialog.message
    Dialog { dialog: usize },
    /// the pointer moved to the element's middle and left there (T3: hover)
    Hover { hover: String },
    /// the system's appearance (T3: Tailwind's dark:)
    Dark { dark: bool },
    /// the window's width at the same height (T3: a breakpoint)
    Width { width: f32 },
}

/// `steps` as JSON: an array of steps.
pub fn parse(json: &str) -> Result<Vec<Step>, String> {
    serde_json::from_str(json).map_err(|e| format!("the script: {e}"))
}

/// DOM's key name as Rune's key.
pub fn key(name: &str) -> rune::Key {
    use rune::Key;
    match name {
        "Enter" => Key::Enter,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Escape" => Key::Escape,
        "Tab" => Key::Tab,
        t => Key::Text(t.into()),
    }
}

/// `step` played on `app`.
pub fn play(app: &mut App, step: &Step) -> Result<(), String> {
    let el = |app: &App, t: &str| app.session.tid(t).ok_or_else(|| format!("no element with data-testid {t:?}"));
    match step {
        Step::Press { press } => {
            let id = el(app, press)?;
            app.input(|s| { let _ = s.click_el(id); })
        }
        Step::Type { r#type, into } => {
            let id = el(app, into)?;
            app.input(|s| { let _ = s.click_el(id); })?;
            app.input(|s| s.key(rune::Key::End))?;
            for c in r#type.chars() {
                app.input(|s| s.key(rune::Key::Text(c.to_string())))?;
            }
            Ok(())
        }
        Step::Wheel { wheel, dy } => {
            let id = el(app, wheel)?;
            let r = app.session.rect(id).ok_or("the element has no box")?;
            app.input(|s| s.wheel(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0, *dy))
        }
        Step::Key { key: k } => app.input(|s| s.key(key(k))),
        Step::Frames { frames } => {
            for _ in 0..*frames {
                app.frame()?;
            }
            Ok(())
        }
        Step::Pick { pick } => queue(raea_io_os::Scripted::Pick(pick.iter().map(Into::into).collect())),
        Step::Save { save } => queue(raea_io_os::Scripted::Save(save.into())),
        Step::Dialog { dialog } => queue(raea_io_os::Scripted::Dialog(*dialog)),
        Step::Hover { hover } => {
            let id = el(app, hover)?;
            app.input(|s| { let _ = s.hover_el(id); })
        }
        Step::Dark { dark } => app.input(|s| s.set_dark(*dark)),
        Step::Width { width } => app.input(|s| { let h = s.viewport().1; s.resize(*width, h) }),
    }
}

fn queue(s: raea_io_os::Scripted) -> Result<(), String> {
    raea_io_os::script(s);
    Ok(())
}
