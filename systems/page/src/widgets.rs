//! Custom widgets (docs/sdk.feature S6d): an element tag backed by a blitz-dom [`Widget`], so code written for Blitz
//! plugs in as Blitz has it, and TSX uses the tag as an element. A plugin (declared in raea.plugins as any Bevy crate
//! is) registers its tag in its build:
//!
//!   app.page_widget("x-gauge", || Box::new(Gauge::default()));
//!
//!   <x-gauge value={0.4} style={{ width: 120, height: 24 }} />
//!
//! (React's ops pass an unknown tag and its props through as an element and its attributes; the plugin's TypeScript
//! types the tag with its own `declare module "react"` IntrinsicElements entry, as sdk/ui's elements.ts does view3d.)
//!
//! Each page element with a registered tag gets its own widget, once, before the page restyles: it hears
//! `connected`, then its attributes as they are (`attribute_changed` from None), and after that blitz-dom's own calls
//! (an attribute set, the element removed). blitz-paint asks it for its scene at the element's box each paint, in
//! the page's order.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use blitz_dom::{BaseDocument, Widget};

use crate::Docs;

/// What makes a tag's widget
pub type MakeWidget = Arc<dyn Fn() -> Box<dyn Widget> + Send + Sync>;

/// The App's widget tags (lowercase, as the page's HTML names them)
#[derive(Resource, Default, Clone)]
pub struct PageWidgets(pub HashMap<String, MakeWidget>);

/// Registering a widget tag on an App
pub trait AppPageWidgets {
    /// `tag`'s elements each backed by a widget `make` makes
    fn page_widget(&mut self, tag: &str, make: impl Fn() -> Box<dyn Widget> + Send + Sync + 'static) -> &mut Self;
}

impl AppPageWidgets for App {
    fn page_widget(&mut self, tag: &str, make: impl Fn() -> Box<dyn Widget> + Send + Sync + 'static) -> &mut Self {
        self.init_resource::<PageWidgets>();
        self.world_mut().resource_mut::<PageWidgets>().0.insert(tag.to_ascii_lowercase(), Arc::new(make));
        self
    }
}

/// What the App's declared Blitz extensions add to every page App (docs/sdk.feature S8b): a raea.plugins entry's
/// "widgets" are registered here once a process (raea-bevy-plugins' install_all, before any App is made), and each
/// PaintPlugin runs them, so the render's and play's page Apps get them as a plain App's page does
static PAGE_PLUGINS: std::sync::Mutex<Vec<fn(&mut App)>> = std::sync::Mutex::new(Vec::new());

/// `install` run in every page App made after this
pub fn register_page_plugin(install: fn(&mut App)) {
    PAGE_PLUGINS.lock().unwrap().push(install);
}

/// The registered page plugins run in `app` (PaintPlugin's build)
pub(crate) fn install_page_plugins(app: &mut App) {
    let installs = PAGE_PLUGINS.lock().unwrap().clone();
    for install in installs {
        install(app);
    }
}

/// Each page's elements with a registered tag given their widgets (PageSystems::Commands, before the restyle)
pub(crate) fn attach(mut docs: NonSendMut<Docs>, widgets: Option<Res<PageWidgets>>) {
    let Some(widgets) = widgets.filter(|w| !w.0.is_empty()) else { return };
    for doc in docs.0.values_mut() {
        attach_doc(doc, &widgets);
    }
}

/// `doc`'s elements with a registered tag and no widget yet given one
pub fn attach_doc(doc: &mut BaseDocument, widgets: &PageWidgets) {
    // the document's elements in tree order, each with the widget for its tag and its attributes as they are
    let mut wanted = Vec::new();
    doc.visit(|id, node| {
        let Some(el) = node.element_data().filter(|el| el.custom_widget_data().is_none()) else { return };
        if let Some(make) = widgets.0.get(&*el.name.local) {
            let attrs: Vec<(String, String)> = el.attrs().iter().map(|a| (a.name.local.to_string(), a.value.clone())).collect();
            wanted.push((id, make.clone(), attrs));
        }
    });
    for (id, make, attrs) in wanted {
        let mut widget = make();
        widget.connected();
        for (name, value) in &attrs {
            widget.attribute_changed(name, None, Some(value));
        }
        doc.mutate().set_custom_widget(id, widget);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyrender::Scene;
    use blitz_dom::{DocumentConfig, LocalName, QualName, ns};
    use std::sync::Mutex;

    /// A swatch, telling what it hears and the box it's painted at
    struct Swatch(Arc<Mutex<Vec<String>>>);
    impl Widget for Swatch {
        fn attribute_changed(&mut self, name: &str, _old: Option<&str>, new: Option<&str>) {
            self.0.lock().unwrap().push(format!("{name}={}", new.unwrap_or("")));
        }
        fn paint(&mut self, _: &mut dyn anyrender::RenderContext, _: &blitz_dom::node::ComputedStyles, w: u32, h: u32, _: f64) -> Scene {
            self.0.lock().unwrap().push(format!("paint {w}x{h}"));
            Scene::new()
        }
    }

    fn q(local: &str) -> QualName {
        QualName { prefix: None, ns: ns!(html), local: LocalName::from(local) }
    }

    /// A registered tag's element, made by the ops TSX's React sends (`<x-swatch color="red" style={...} />`), gets
    /// one widget, which hears its attributes as they are and then a prop's change, and is painted at the element's
    /// box; an element of another tag gets none
    #[test]
    fn a_registered_tag_is_backed_by_its_widget() {
        use raea_core_wire::Op;
        use serde_json::json;
        let heard = Arc::new(Mutex::new(Vec::new()));
        let mut widgets = PageWidgets::default();
        let h = heard.clone();
        widgets.0.insert("x-swatch".into(), Arc::new(move || Box::new(Swatch(h.clone())) as Box<dyn Widget>));
        let mut doc = BaseDocument::new(DocumentConfig::default());
        let body = {
            let mut m = doc.mutate();
            let root = m.doc.root_node().id;
            let html = m.create_element(q("html"), vec![]);
            m.append_children(root, &[html]);
            let body = m.create_element(q("body"), vec![]);
            m.append_children(html, &[body]);
            body
        };
        let mut nodes = crate::ops::Nodes::new(body);
        let props = |v: serde_json::Value| v.as_object().unwrap().clone();
        crate::ops::apply(&mut doc, &mut nodes, vec![
            Op::Create { id: 1, kind: "x-swatch".into(), props: props(json!({"color": "red", "style": {"display": "block", "width": 30, "height": 20}})), text: None },
            Op::Create { id: 2, kind: "x-other".into(), props: Default::default(), text: None },
            Op::Append { parent: 0, child: 1 },
            Op::Append { parent: 0, child: 2 },
        ]);
        let (swatch, plain) = (nodes.by_id[&1], nodes.by_id[&2]);
        attach_doc(&mut doc, &widgets);
        attach_doc(&mut doc, &widgets);
        assert_eq!(doc.custom_widget_node_ids(), vec![swatch]);
        assert!(doc.get_node(plain).and_then(|n| n.element_data()).unwrap().custom_widget_data().is_none());
        crate::ops::apply(&mut doc, &mut nodes, vec![Op::Update { id: 1, set: Some(props(json!({"color": "blue"}))), unset: None, text: None }]);
        doc.resolve(0.0);
        let mut scene = Scene::new();
        blitz_paint::paint_scene(&mut scene, &mut doc, 1.0, 100, 100, 0, 0);
        let got: Vec<String> = heard.lock().unwrap().iter().filter(|h| !h.starts_with("style=")).cloned().collect();
        assert_eq!(got, ["color=red", "color=blue", "paint 30x20"]);
    }
}
