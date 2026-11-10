//! Blitz's extension points as Bevy resources (docs/sdk.feature S6d): what a document is made with (blitz-dom's
//! DocumentConfig), read from the App when [`open_doc`](crate::open_doc) makes one, so code written for Blitz or Dioxus
//! Native plugs in by inserting its provider as it would hand it to a document:
//!
//! - [`PageNet`]: the net provider, which fetches a page's resources (images, stylesheets, fonts). PaintPlugin
//!   inserts raea's FileNet (files beside the page and under RAEA_PAGE_ROOT) unless the App has one.
//! - [`PageNavigation`]: the navigation provider, which hears a link's click and a form's submit. None by default.
//! - [`PageHtmlParser`]: the HTML parser setInnerHTML and an <iframe>'s document use. PaintPlugin inserts
//!   blitz-html's while iframes are on (RAEA_PAGE_IFRAMES).
//! - [`PageFonts`]: the font collection (B0d's: one a process, shared with bevy_text). A document given faces of its
//!   own keeps them.
//! - [`PageShell`]: the shell provider, which holds the clipboard a text field's Cmd+C, Cmd+X and Cmd+V use. Blitz's
//!   default has none (a paste inserted nothing); raea-page-host inserts one on raea-io-os's clipboard.
//!
//! Each takes effect for the documents made after it's inserted; a document already open keeps what it was made with,
//! as a Blitz document does.

use std::sync::Arc;

use bevy::prelude::*;
use blitz_dom::{DocumentConfig, FontContext, HtmlParserProvider};
use blitz_traits::navigation::NavigationProvider;
use blitz_traits::net::NetProvider;
use blitz_traits::shell::ShellProvider;

use crate::Page;

/// The net provider a page's document fetches its resources through
#[derive(Resource, Clone)]
pub struct PageNet(pub Arc<dyn NetProvider>);

/// The navigation provider a page's document sends a link's click and a form's submit to
#[derive(Resource, Clone)]
pub struct PageNavigation(pub Arc<dyn NavigationProvider>);

/// The HTML parser a page's document parses with (setInnerHTML, an <iframe>'s document)
#[derive(Resource, Clone)]
pub struct PageHtmlParser(pub Arc<dyn HtmlParserProvider + Send + Sync>);

/// The shell provider a page's document reaches the system through: the clipboard (a text field's copy, cut and paste)
#[derive(Resource, Clone)]
pub struct PageShell(pub Arc<dyn ShellProvider>);

/// The font collection a page's document draws its text with, unless it's given faces of its own
#[derive(Resource, Clone)]
pub struct PageFonts(pub FontContext);

/// The providers raea's pages have by default, each left as it is when the App already has one
pub(crate) fn defaults(app: &mut App) {
    if !app.world().contains_resource::<PageNet>() {
        app.insert_resource(PageNet(Arc::new(crate::net::FileNet { roots: crate::net::page_roots() })));
    }
    if crate::iframes() && !app.world().contains_resource::<PageHtmlParser>() {
        app.insert_resource(PageHtmlParser(Arc::new(blitz_html::HtmlProvider)));
    }
}

/// A document's config for a page `at` in `world`: [`config`](crate::config) with the App's providers, and `fonts`
/// (the page's own faces) or the App's collection
pub fn config_in(world: &World, at: Page, fonts: Option<FontContext>) -> DocumentConfig {
    let mut config = crate::config(at);
    if let Some(net) = world.get_resource::<PageNet>() {
        config.net_provider = Some(net.0.clone());
    }
    if let Some(nav) = world.get_resource::<PageNavigation>() {
        config.navigation_provider = Some(nav.0.clone());
    }
    if let Some(parser) = world.get_resource::<PageHtmlParser>() {
        config.html_parser_provider = Some(parser.0.clone());
    }
    if let Some(shell) = world.get_resource::<PageShell>() {
        config.shell_provider = Some(shell.0.clone());
    }
    config.font_ctx = fonts;
    if config.font_ctx.is_none()
        && let Some(PageFonts(own)) = world.get_resource::<PageFonts>()
    {
        config.font_ctx = Some(own.clone());
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitz_traits::navigation::NavigationOptions;

    struct Heard(std::sync::Mutex<Vec<String>>);
    impl NavigationProvider for Heard {
        fn navigate_to(&self, options: NavigationOptions) {
            self.0.lock().unwrap().push(options.url.to_string());
        }
    }

    /// A provider inserted as a resource is what the App's next document is made with; one there by default is
    /// PaintPlugin's, and a page's own faces win over the App's
    #[test]
    fn documents_are_made_with_the_apps_providers() {
        let mut app = App::new();
        defaults(&mut app);
        let at = Page { size: UVec2::new(100, 100), scale: 1.0 };
        let config = config_in(app.world(), at, None);
        assert!(config.net_provider.is_some() && config.navigation_provider.is_none() && config.font_ctx.is_none() && config.shell_provider.is_none());

        let nav: Arc<dyn NavigationProvider> = Arc::new(Heard(Default::default()));
        app.insert_resource(PageNavigation(nav.clone()));
        app.insert_resource(PageFonts(FontContext::default()));
        let shell: Arc<dyn ShellProvider> = Arc::new(blitz_traits::shell::DummyShellProvider);
        app.insert_resource(PageShell(shell.clone()));
        let config = config_in(app.world(), at, None);
        assert!(Arc::ptr_eq(config.navigation_provider.as_ref().unwrap(), &nav));
        assert!(Arc::ptr_eq(config.shell_provider.as_ref().unwrap(), &shell));
        assert!(config.font_ctx.is_some());

        // defaults leave a provider the App put in first
        let net = app.world().resource::<PageNet>().0.clone();
        defaults(&mut app);
        assert!(Arc::ptr_eq(&app.world().resource::<PageNet>().0, &net));
    }
}
