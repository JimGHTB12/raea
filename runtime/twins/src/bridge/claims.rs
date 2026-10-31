//! Who owns an element's properties (H9): claims by the page or the app, conflicts, reasserting, and commands applied.

use super::*;

/// Who a property of an element belongs to (H9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// React set it (a style property, a class, an attribute or the element's text, through its ops)
    React,
    /// a system set it first (PageCommands)
    Bevy,
    /// the page itself: a twin's own components, `PageElement` (which element it is) and `ElementBox` (its box as laid
    /// out), on every page
    Page,
}

/// A write to a property its owner holds, from the other side (H9): `property` is its CSS name (`background-color`),
/// an attribute as `[name]`, a class as `.name`, or `text`. The owner's value stays: a system's write to React's is
/// refused, and React's to Bevy's is put back before restyle. Logged as an error too.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct PageConflict {
    pub twin: Entity,
    pub property: String,
    pub owner: Owner,
}

/// What systems own on React pages' elements, by twin.
#[derive(Resource, Default)]
pub struct Claims(HashMap<Entity, Claim>);

#[derive(Default, Debug)]
pub(crate) struct Claim {
    /// style properties, by the name the system gave
    pub(crate) style: BTreeMap<String, String>,
    pub(crate) attributes: BTreeMap<String, String>,
    /// classes added, each with whether React had it too (removing it then leaves it, as React's)
    pub(crate) classes: BTreeMap<String, bool>,
    /// the text node a system made the element's content, and its text
    pub(crate) text: Option<(NodeId, String)>,
    /// the element's style block as the claims last left it: another means React's commit parsed the style attribute
    /// again (blitz-dom keeps a set property in the block, not the attribute), and the claims are set again
    pub(crate) block: usize,
    /// the class attribute as the claims last left it
    pub(crate) class: Option<String>,
}

/// The queued changes applied to their elements' documents; a change to a twin that's gone is dropped. On a React
/// page, what React's commit since did to Bevy's properties is put right first, then each change is checked against
/// React's.
pub(crate) fn apply_commands(mut docs: NonSendMut<Docs>, mut queue: ResMut<PageQueue>, mut claims: ResMut<Claims>, elements: Query<&PageElement>, rendered: Query<(), With<Rendered>>, mut conflicts: MessageWriter<PageConflict>) {
    let mut report = |twin: Entity, property: String, owner: Owner| {
        let by = if owner == Owner::React { "a system" } else { "React" };
        error!("raea-page: {by} wrote {property} on {twin}, which is {owner:?}'s; {owner:?}'s value stays (docs/blitz-bevy.feature B3 H9)");
        conflicts.write(PageConflict { twin, property, owner });
    };
    claims.0.retain(|twin, _| elements.contains(*twin));
    for (twin, claim) in claims.0.iter_mut() {
        let Ok(el) = elements.get(*twin) else { continue };
        let Some(doc) = docs.0.get_mut(&el.page) else { continue };
        for property in reassert(doc, el.node, claim) {
            report(*twin, property, Owner::Bevy);
        }
    }
    for (twin, op) in queue.0.drain(..) {
        let Ok(el) = elements.get(twin) else { continue };
        let Some(doc) = docs.0.get_mut(&el.page) else { continue };
        if !rendered.contains(el.page) {
            apply(doc, el.node, op);
            continue;
        }
        if let Err(property) = claim(doc, el.node, claims.0.entry(twin).or_default(), op) {
            report(twin, property, Owner::React);
        }
    }
}

/// `op` on a React page's element, as a claim: refused (the property's name) where it writes what React set.
pub(crate) fn claim(doc: &mut BaseDocument, node: NodeId, claim: &mut Claim, op: PageOp) -> Result<(), String> {
    let Some(n) = doc.get_node(node) else { return Ok(()) };
    let children = n.children.clone();
    let attr = |doc: &BaseDocument, a: &str| doc.get_node(node).and_then(|n| n.attr(blitz_dom::LocalName::from(a))).map(str::to_string);
    let react_style = style_names(attr(doc, "style").as_deref());
    let classes: Vec<String> = attr(doc, "class").unwrap_or_default().split_ascii_whitespace().map(str::to_string).collect();
    match op {
        PageOp::SetStyle(p, v) => {
            if react_style.iter().any(|r| overlaps(r, &p)) {
                return Err(p);
            }
            apply(doc, node, PageOp::SetStyle(p.clone(), v.clone()));
            claim.style.insert(p, v);
        }
        PageOp::RemoveStyle(p) => {
            if claim.style.remove(&p).is_some() {
                apply(doc, node, PageOp::RemoveStyle(p));
            } else if react_style.iter().any(|r| overlaps(r, &p)) {
                return Err(p);
            }
        }
        // (the whole style or class attribute is React's: a system writes properties and classes)
        PageOp::SetAttribute(a, _) | PageOp::RemoveAttribute(a) if matches!(a.as_str(), "style" | "class") => return Err(format!("[{a}]")),
        PageOp::SetAttribute(a, v) => {
            if !claim.attributes.contains_key(&a) && attr(doc, &a).is_some() {
                return Err(format!("[{a}]"));
            }
            apply(doc, node, PageOp::SetAttribute(a.clone(), v.clone()));
            claim.attributes.insert(a, v);
        }
        PageOp::RemoveAttribute(a) => {
            if claim.attributes.remove(&a).is_some() {
                apply(doc, node, PageOp::RemoveAttribute(a));
            } else if attr(doc, &a).is_some() {
                return Err(format!("[{a}]"));
            }
        }
        PageOp::AddClass(c) => {
            if !claim.classes.contains_key(&c) {
                let react = classes.contains(&c);
                apply(doc, node, PageOp::AddClass(c.clone()));
                claim.classes.insert(c, react);
            }
        }
        PageOp::RemoveClass(c) => match claim.classes.remove(&c) {
            Some(false) => apply(doc, node, PageOp::RemoveClass(c)),
            Some(true) => {}
            None if classes.contains(&c) => return Err(format!(".{c}")),
            None => {}
        },
        PageOp::SetText(t) => match claim.text.as_ref().map(|c| c.0) {
            Some(tn) if children == [tn] => {
                doc.mutate().set_node_text(tn, &t);
                claim.text = Some((tn, t));
            }
            // (the element's content is React's)
            _ if !children.is_empty() => return Err("text".into()),
            _ => {
                let mut m = doc.mutate();
                let tn = m.create_text_node(&t);
                m.append_children(node, &[tn]);
                claim.text = Some((tn, t));
            }
        },
    }
    claim.block = block(doc, node);
    claim.class = attr(doc, "class");
    Ok(())
}

/// A system's claims on `node` set again where React's commit undid them; the properties React wrote that were Bevy's.
pub(crate) fn reassert(doc: &mut BaseDocument, node: NodeId, claim: &mut Claim) -> Vec<String> {
    let Some(n) = doc.get_node(node) else { return vec![] };
    let mut wrote = vec![];
    let attr = |a: &str| n.attr(blitz_dom::LocalName::from(a)).map(str::to_string);
    let (style, class, children) = (attr("style"), attr("class"), n.children.clone());
    let attributes: Vec<(String, String)> = claim.attributes.iter().filter(|(a, v)| attr(a).as_ref() != Some(*v)).map(|(a, v)| (a.clone(), v.clone())).collect();
    if !claim.style.is_empty() && block(doc, node) != claim.block {
        let react = style_names(style.as_deref());
        for (p, v) in &claim.style {
            if react.iter().any(|r| overlaps(r, p)) {
                wrote.push(p.clone());
            }
            doc.mutate().set_style_property(node, p, v);
        }
        claim.block = block(doc, node);
    }
    if !claim.classes.is_empty() && class != claim.class {
        for c in claim.classes.keys() {
            apply(doc, node, PageOp::AddClass(c.clone()));
        }
        claim.class = doc.get_node(node).and_then(|n| n.attr(blitz_dom::LocalName::from("class"))).map(str::to_string);
    }
    for (a, v) in attributes {
        wrote.push(format!("[{a}]"));
        apply(doc, node, PageOp::SetAttribute(a, v));
    }
    if let Some((tn, t)) = claim.text.clone() {
        if children != [tn] {
            // React gave the element content of its own: that stays (its nodes are its ops'), and the claim goes
            wrote.push("text".into());
            claim.text = None;
        } else if doc.get_node(tn).is_some_and(|x| x.text_content() != t) {
            wrote.push("text".into());
            doc.mutate().set_node_text(tn, &t);
        }
    }
    wrote
}

/// The element's style block, as an address (blitz-dom makes a new one when the style attribute is parsed).
pub(crate) fn block(doc: &BaseDocument, node: NodeId) -> usize {
    doc.get_node(node).and_then(|n| n.element_data()).and_then(|e| e.style_attribute.as_ref()).map(|b| &**b as *const _ as usize).unwrap_or(0)
}

/// The property names a style attribute sets (React's: ops.rs's style_css writes `name: value;`).
pub(crate) fn style_names(style: Option<&str>) -> Vec<String> {
    style.unwrap_or("").split(';').filter_map(|d| d.split_once(':')).map(|(k, _)| k.trim().to_ascii_lowercase()).filter(|k| !k.is_empty()).collect()
}

/// Whether two CSS properties set any of the same longhands (`background` and `background-color` do).
pub(crate) fn overlaps(a: &str, b: &str) -> bool {
    fn longhands(name: &str) -> Vec<style::properties::LonghandId> {
        match style::properties::PropertyId::parse_enabled_for_all_content(name) {
            Ok(style::properties::PropertyId::NonCustom(id)) => {
                let id = id.unaliased();
                id.as_longhand().map(|l| vec![l]).or_else(|| id.as_shorthand().map(|s| s.longhands().collect())).unwrap_or_default()
            }
            _ => vec![],
        }
    }
    if a.eq_ignore_ascii_case(b) {
        return true;
    }
    let la = longhands(a);
    longhands(b).iter().any(|l| la.contains(l))
}

pub(crate) fn apply(doc: &mut BaseDocument, node: NodeId, op: PageOp) {
    let class = |doc: &BaseDocument| doc.get_node(node).and_then(|n| n.attr(blitz_dom::LocalName::from("class"))).unwrap_or("").to_string();
    match op {
        PageOp::SetStyle(p, v) => doc.mutate().set_style_property(node, &p, &v),
        PageOp::RemoveStyle(p) => doc.mutate().remove_style_property(node, &p),
        PageOp::SetAttribute(a, v) => doc.mutate().set_attribute(node, raea_sys_page::ops::qual(&a), &v),
        PageOp::RemoveAttribute(a) => doc.mutate().clear_attribute(node, raea_sys_page::ops::qual(&a)),
        PageOp::AddClass(c) => {
            let have = class(doc);
            if !have.split_ascii_whitespace().any(|h| h == c) {
                let to = if have.trim().is_empty() { c } else { format!("{} {c}", have.trim()) };
                doc.mutate().set_attribute(node, raea_sys_page::ops::qual("class"), &to);
            }
        }
        PageOp::RemoveClass(c) => {
            let have = class(doc);
            if have.split_ascii_whitespace().any(|h| h == c) {
                let to = have.split_ascii_whitespace().filter(|h| *h != c).collect::<Vec<_>>().join(" ");
                doc.mutate().set_attribute(node, raea_sys_page::ops::qual("class"), &to);
            }
        }
        PageOp::SetText(t) => {
            let mut m = doc.mutate();
            m.remove_and_drop_all_children(node);
            let text = m.create_text_node(&t);
            m.append_children(node, &[text]);
        }
    }
}
