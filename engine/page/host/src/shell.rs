//! The shell a page's documents are made with (raea-sys-page's PageShell): the clipboard, so a text field's Cmd+C,
//! Cmd+X and Cmd+V reach it. Blitz's default shell has none, so a paste into any field of raea.app inserted nothing
//! (Shane 11-09: "make sure i can paste text into any text field"). It's raea-io-os's clipboard: the system's in the
//! window, the process's own headless, as navigator.clipboard's is.

use blitz_traits::shell::{ClipboardError, ShellProvider};

/// Blitz's shell for raea's pages: the clipboard, and the rest left as Blitz's default has them
pub struct Shell;

impl ShellProvider for Shell {
    fn get_clipboard_text(&self) -> Result<String, ClipboardError> {
        raea_io_os::clipboard_text().map_err(|_| ClipboardError)
    }

    fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
        raea_io_os::set_clipboard_text(text).map_err(|_| ClipboardError)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::prelude::*;
    use blitz_dom::{BaseDocument, EventDriver, NoopEventHandler};
    use blitz_traits::NodeId;
    use blitz_traits::events::{BlitzKeyEvent, KeyState, UiEvent};
    use keyboard_types::{Code, Key, Location, Modifiers};
    use raea_sys_page::{Page, PageShell};

    /// A page with one field, made with the App's providers and `shell`, the field focused.
    fn field(shell: Option<PageShell>) -> (BaseDocument, NodeId) {
        let mut app = App::new();
        if let Some(s) = shell {
            app.insert_resource(s);
        }
        let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
        let config = raea_sys_page::providers::config_in(app.world(), at, None);
        let html = r#"<body style="margin:0"><input id="f" value="ab" style="display:block;width:200px;height:30px"></body>"#;
        let mut doc = blitz_html::HtmlDocument::from_html(html, config).into_inner();
        doc.resolve(0.0);
        let f = doc.get_element_by_id("f").unwrap();
        doc.set_focus_to(f);
        (doc, f)
    }

    /// Cmd and `c` pressed, as raea's window hands Blitz a key (runtime/dom bridge.rs WindowInput: Super is META)
    fn cmd(doc: &mut BaseDocument, c: &str) {
        let ev = BlitzKeyEvent { key: Key::Character(c.into()), code: Code::KeyV, modifiers: Modifiers::META, location: Location::Standard, is_auto_repeating: false, is_composing: false, state: KeyState::Pressed, text: None };
        EventDriver::new(doc, NoopEventHandler).handle_ui_event(UiEvent::KeyDown(ev));
    }

    fn value(doc: &BaseDocument, f: NodeId) -> String {
        doc.get_node(f).unwrap().element_data().unwrap().text_input_data().unwrap().editor.raw_text().to_string()
    }

    /// Cmd+V in a field pastes the clipboard's text at the caret, and Cmd+A, Cmd+C copies the field's; without
    /// the shell (Blitz's default) the paste inserts nothing, which raea.app's fields did before
    #[test]
    fn a_field_pastes_and_copies_through_the_clipboard() {
        // the process's own clipboard: the test never touches the Mac's
        raea_io_os::set_headless();
        raea_io_os::set_clipboard_text("sunmelt".into()).unwrap();

        let (mut bare, f) = field(None);
        cmd(&mut bare, "v");
        assert_eq!(value(&bare, f), "ab", "Blitz's default shell has no clipboard");

        let (mut doc, f) = field(Some(PageShell(Arc::new(super::Shell))));
        cmd(&mut doc, "v");
        // focused from code, the caret is at the field's start
        assert_eq!(value(&doc, f), "sunmeltab");

        cmd(&mut doc, "a");
        cmd(&mut doc, "c");
        raea_io_os::set_clipboard_text(String::new()).unwrap();
        cmd(&mut doc, "c");
        assert_eq!(raea_io_os::clipboard_text().unwrap(), "sunmeltab");
    }
}
