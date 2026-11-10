//! WorldPagePlugin: the page drawn in an App's own World, with its screens, videos and 3D slot.

use super::*;

/// The composition's page in the App's own World (docs/blitz-bevy.feature B3 H5, sdk.feature S6d): beside ScenePlugin,
/// a plain Bevy App draws the composition's HTML layer too, on Blitz, with no second App. Each Update, after
/// ScenePlugin's rows, the React tree (its <AbsoluteFill>s, divs, text and styles, the 3D's holders see-through, the
/// space's brand kit first) is diffed into a Blitz document in this World, raea-page's PagePlugin restyles and paints
/// it, and it's laid over the scene's camera as raea's feed lays it (raea-page's PageOverlay: the same bytes, B3
/// H10), with a UI node the page's size holding its PageView, so the page is in Bevy's picking. Its CSS clock is the
/// scene's frame.
/// Its <video>s show their frames at their source times, decoded as raea-page's BlitzPage decodes them.
/// Its <Screen>s' nested compositions are documents of their own, copied onto their screens in the 3D.
/// A layered Canvas3D's 3D is drawn in its slot, between the page's elements (world_page_show).
/// Its texture elements show the App's PageTextures.
pub struct WorldPagePlugin {
    pub size: (u32, u32),
    /// where the composition's media resolve (its folder)
    pub dir: PathBuf,
    pub fps: f64,
}

#[derive(Resource)]
pub struct WorldPage {
    pub page: Entity,
    pub(crate) tree: raea_core_wire::tree::Tree,
    pub(crate) nodes: raea_sys_page::ops::Nodes,
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) dir: PathBuf,
    pub(crate) brand: Option<String>,
    pub(crate) fps: f64,
    /// each <Screen>'s nested composition, by the screen's name
    pub(crate) screens: HashMap<String, WorldScreen>,
    /// a layered Canvas3D's slot (its element's id), as the last walk found it, and the image its 3D is drawn into
    pub(crate) slot: Option<u64>,
    pub(crate) slot_image: Option<Handle<Image>>,
    /// the texture elements (`data-raea-texture`) the last walk found: each one's texture id and its element's id
    pub(crate) texture_slots: Vec<(u64, u64)>,
    /// the last walk's elements, kept for the next
    pub(crate) walk: Walk,
}

/// Textures a plain App's page shows by id: every `<img data-raea-texture={id}>` shows image `id` (a camera's target,
/// any Bevy Image), GPU to GPU, as live play's share_texture does on its page.
#[derive(Resource, Default)]
pub struct PageTextures(pub HashMap<u64, Handle<Image>>);

/// A <Screen>'s nested composition in a plain App: a document of its own at the screen's size (BlitzPage's sync_sub),
/// painted by PagePlugin into its own image, which is copied into raea-render's screen of that name each frame
/// (`copy_screens`, GPU to GPU, as Engine::copy_screen does in live play).
pub(crate) struct WorldScreen {
    pub(crate) page: Entity,
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) tree: raea_core_wire::tree::Tree,
    pub(crate) nodes: raea_sys_page::ops::Nodes,
}

/// Each screen's page image and the screen's image it's copied into, this frame.
#[derive(Resource, Default, Clone)]
pub(crate) struct ScreenCopies(Vec<(AssetId<Image>, AssetId<Image>)>);

/// The page's video decoders (not Send: a World's own, beside WorldPage).
#[derive(Default)]
pub(crate) struct WorldVideos(raea_page::motion::Videos);

impl Plugin for WorldPagePlugin {
    fn build(&self, app: &mut App) {
        let (w, h) = self.size;
        let fonts = raea_sys_fonts::context(&raea_io_home::home().join("core/fonts"));
        let (page, nodes) = raea_sys_page::open_doc(app.world_mut(), w, h, Some(fonts));
        let brand = raea_runtime_react::scene::brand_css(&self.dir, w);
        app.insert_resource(WorldPage { page, tree: Default::default(), nodes, w, h, dir: self.dir.clone(), brand, fps: self.fps, screens: HashMap::new(), slot: None, slot_image: None, texture_slots: Vec::new(), walk: Walk::default() })
            .init_resource::<PageTextures>()
            .init_non_send::<WorldVideos>()
            .init_resource::<ScreenCopies>()
            .add_systems(PostUpdate, screen_copies.after(raea_sys_page::PageSystems::Paint))
            .add_systems(Update, world_page_sync.after(raea_runtime_react::scene::SceneSystems::Rows))
            .add_systems(PostUpdate, world_page_show.before(raea_sys_page::PageSystems::Commands))
            // the input log's frame is the scene's (B3 H10: a play recorded, a render replayed, by the same index)
            .add_systems(First, input_frame.after(raea_runtime_react::scene::SceneSystems::Frame).before(bevy::picking::PickingSystems::PostInput));
        if !app.is_plugin_added::<raea_runtime_input::replay::ReplayPlugin>() {
            app.add_plugins(raea_runtime_input::replay::ReplayPlugin);
        }
        handlers(app);
        world_input::install(app);
        if let Some(r) = app.get_sub_app_mut(bevy::render::RenderApp) {
            r.add_systems(bevy::render::ExtractSchedule, extract_copies).add_systems(bevy::render::Render, copy_screens.in_set(bevy::render::RenderSystems::PrepareBindGroups));
        }
    }
}

/// The pages whose elements are React's own (bevy-react's ReactNode ids as their twins' ops): WorldPage's and its
/// <Screen>s', as its last sync left them.
#[derive(Resource, Default)]
pub struct ReactPages(pub Vec<Entity>);

/// The page's handler step (B3 H8): each DOM event Blitz fires on an element of this page (or a <Screen>'s), as the
/// bridge delivers it to the element's twin and each twin up its path, goes to the React element's `on*` prop, as
/// sdk/react pointer.rs sends an entity's: `{"t": "uiEvent", "event": {id, kind, ...}}`, so React's handlers run on
/// the page as they do on its entities. React commits what they set at its next sync, a frame later, as a live
/// host's handlers do.
pub(crate) fn handlers(app: &mut App) {
    use raea_runtime_twins::bridge::dom;
    app.init_resource::<ReactPages>();
    use serde_json::json;
    fn pointer(p: &dom::BlitzPointerEvent) -> Map<String, Value> {
        let button = match p.button {
            dom::MouseEventButton::Main => 0,
            dom::MouseEventButton::Auxiliary => 1,
            dom::MouseEventButton::Secondary => 2,
            _ => 0,
        };
        let mut m = Map::new();
        m.insert("x".into(), json!(p.coords.page_x));
        m.insert("y".into(), json!(p.coords.page_y));
        m.insert("button".into(), json!(button));
        m
    }
    macro_rules! route {
        ($($ty:ident => $kind:literal, $data:expr;)*) => {$(
            app.add_observer(|ev: On<dom::$ty>, twins: Query<&raea_runtime_twins::PageElement>, react: Query<(&ReactNode, &ReactProps)>, pages: Res<ReactPages>, out: Option<Res<raea_runtime_react::ReactOutbox>>| {
                let Some(out) = out else { return };
                let Ok(el) = twins.get(ev.entity) else { return };
                if !pages.0.contains(&el.page) {
                    return;
                }
                let Some(op) = el.op else { return };
                let prop = concat!("on", $kind);
                let Some(_) = react.iter().find(|(n, p)| n.0 == op && p.0.get(prop) == Some(&Value::Bool(true))) else { return };
                let mut m: Map<String, Value> = ($data)(&*ev);
                m.insert("id".into(), json!(op));
                let k = &prop[2..];
                m.insert("kind".into(), json!(format!("{}{}", k[..1].to_lowercase(), &k[1..])));
                out.send(json!({"t": "uiEvent", "event": m}));
            });
        )*};
    }
    route! {
        Click => "Click", |e: &dom::Click| pointer(&e.data);
        DoubleClick => "DoubleClick", |e: &dom::DoubleClick| pointer(&e.data);
        ContextMenu => "ContextMenu", |e: &dom::ContextMenu| pointer(&e.data);
        PointerDown => "PointerDown", |e: &dom::PointerDown| pointer(&e.data);
        PointerUp => "PointerUp", |e: &dom::PointerUp| pointer(&e.data);
        PointerMove => "PointerMove", |e: &dom::PointerMove| pointer(&e.data);
        PointerEnter => "PointerEnter", |e: &dom::PointerEnter| pointer(&e.data);
        PointerLeave => "PointerLeave", |e: &dom::PointerLeave| pointer(&e.data);
        PointerOver => "PointerOver", |e: &dom::PointerOver| pointer(&e.data);
        PointerOut => "PointerOut", |e: &dom::PointerOut| pointer(&e.data);
        PointerCancel => "PointerCancel", |e: &dom::PointerCancel| pointer(&e.data);
        Wheel => "Wheel", |e: &dom::Wheel| {
            let (dx, dy) = match e.data.delta {
                dom::BlitzWheelDelta::Lines(x, y) | dom::BlitzWheelDelta::Pixels(x, y) => (x, y),
            };
            let mut m = Map::new();
            m.insert("x".into(), json!(e.data.coords.page_x));
            m.insert("y".into(), json!(e.data.coords.page_y));
            m.insert("deltaX".into(), json!(-dx));
            m.insert("deltaY".into(), json!(-dy));
            m
        };
        KeyDown => "KeyDown", |e: &dom::KeyDown| Map::from_iter([("key".to_string(), json!(e.data.key.to_string()))]);
        KeyUp => "KeyUp", |e: &dom::KeyUp| Map::from_iter([("key".to_string(), json!(e.data.key.to_string()))]);
        // React's onChange is the DOM's input event
        Input => "Change", |e: &dom::Input| Map::from_iter([("value".to_string(), json!(e.data.value))]);
        Focus => "Focus", |_: &dom::Focus| Map::new();
        Blur => "Blur", |_: &dom::Blur| Map::new();
    }
}

pub(crate) fn input_frame(scene: Option<Res<raea_runtime_react::scene::SceneFrame>>, mut f: ResMut<raea_runtime_input::replay::InputFrame>) {
    if let Some(s) = scene {
        f.0 = s.0;
    }
}

/// The page made to match the React tree as it stands this frame (BevyPage::sync's walk, into this World's document).
pub fn world_page_sync(world: &mut World) {
    let Some(mut wp) = world.remove_resource::<WorldPage>() else { return };
    // the 3D's holders see through, as every path that draws the 3D under the page has them (a layered Canvas3D's 3D
    // is in the page's order instead, and its holders stay as they are, as render's and live play's)
    let mut held = Vec::new();
    if raea_runtime_react::scene::scene_slot(world).is_none() {
        raea_runtime_react::scene::holder_entities(world, &mut held);
    }
    let mut holders = HashMap::new();
    for e in held {
        let (Some(n), Some(p)) = (world.get::<raea_runtime_react::ReactNode>(e), world.get::<ReactProps>(e)) else { continue };
        let mut p = p.0.clone();
        raea_runtime_react::scene::see_through(&mut p);
        holders.insert(n.0, p);
    }
    if let Some(surf) = raea_runtime_react::scene::surface(world) {
        let (mut videos, mut screens) = (Vec::new(), Vec::new());
        SCENE_SLOT.with(|c| c.set(None));
        IN_3D.with(|c| c.set(0));
        IN_SVG.with(|c| c.set(false));
        TEXTURE_SLOTS.with(|c| c.borrow_mut().clear());
        // a UiMaterial needs raea-render's engine (BevyPage's hosts); here its element is a box of nothing
        UI_MATERIALS.with(|c| c.borrow_mut().clear());
        REFUSED.with(|c| c.borrow_mut().clear());
        let mut w = std::mem::take(&mut wp.walk);
        w.begin(wp.w, wp.h, wp.brand.as_deref(), false);
        walk(world, surf, BODY, At::Str(""), &wp.dir, &holders, &mut w, &mut videos, &mut screens);
        w.end();
        // a plain App's system has no render to fail: it says so (the walk meets an element's props again only when they
        // change)
        for why in REFUSED.with(|c| std::mem::take(&mut *c.borrow_mut())) {
            eprintln!("raea: {why}");
        }
        wp.slot = SCENE_SLOT.with(|c| c.get());
        wp.texture_slots = TEXTURE_SLOTS.with(|c| std::mem::take(&mut *c.borrow_mut()));
        let mut decoded = world.remove_non_send::<WorldVideos>().unwrap_or_default();
        sync_doc(world, wp.page, &mut wp.tree, &mut wp.nodes, w.wants(), &w.order, videos, &mut decoded.0);
        wp.walk = w;
        // each <Screen>'s nested composition: a document of its own at the screen's size (BevyPage::sync's)
        let mut keep = Vec::new();
        for s in screens {
            let (mut want, mut order, first) = frame_wants(s.w, s.h, wp.brand.as_deref());
            want.extend(s.want);
            order.extend(s.order);
            order.insert(BODY, first.into_iter().chain(std::iter::once(s.top)).collect());
            if wp.screens.get(&s.name).is_some_and(|x| (x.w, x.h) != (s.w, s.h)) {
                let gone = wp.screens.remove(&s.name).unwrap();
                drop_doc(world, gone.page);
            }
            let sub = wp.screens.entry(s.name.clone()).or_insert_with(|| {
                let fonts = raea_sys_fonts::context(&raea_io_home::home().join("core/fonts"));
                let (page, nodes) = raea_sys_page::open_doc(world, s.w, s.h, Some(fonts));
                WorldScreen { page, w: s.w, h: s.h, tree: Default::default(), nodes }
            });
            sync_doc(world, sub.page, &mut sub.tree, &mut sub.nodes, &want, &order, s.videos, &mut decoded.0);
            keep.push(s.name);
        }
        wp.screens.retain(|name, s| {
            let kept = keep.contains(name);
            if !kept {
                drop_doc(world, s.page);
            }
            kept
        });
        world.insert_non_send(decoded);
    }
    if let Some(mut p) = world.get_resource_mut::<ReactPages>() {
        p.0 = std::iter::once(wp.page).chain(wp.screens.values().map(|s| s.page)).collect();
    }
    // the page's CSS animations at the scene's frame
    if let Some(f) = world.get_resource::<raea_runtime_react::scene::SceneFrame>() {
        let at = f.0 as f64 / wp.fps;
        world.resource_mut::<raea_sys_page::PageClock>().0 = Some(at);
    }
    world.insert_resource(wp);
}

/// A document brought to `want` (the tree's diff applied), and its <video>s at their frames.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_doc(world: &mut World, page: Entity, tree: &mut raea_core_wire::tree::Tree, nodes: &mut raea_sys_page::ops::Nodes, want: &[Want], order: &HashMap<u64, Vec<u64>>, videos: Vec<VideoAt>, decoded: &mut raea_page::motion::Videos) {
    let (ops, _) = tree.diff(want, order);
    let mut docs = world.non_send_mut::<raea_sys_page::Docs>();
    let Some(doc) = docs.0.get_mut(&page) else { return };
    if !ops.is_empty() {
        raea_sys_page::ops::apply(doc, nodes, ops);
        // each React element's twin keyed by its React id (H1's useEntity(ref)); the page's own elements have none
        let ids = nodes.by_id.iter().filter(|(id, _)| **id >= REACT_IDS).map(|(id, n)| (*n, id - REACT_IDS)).collect();
        drop(docs);
        if let Some(mut t) = world.get_resource_mut::<raea_runtime_twins::bridge::TwinOps>() {
            t.0.insert(page, ids);
        }
        docs = world.non_send_mut::<raea_sys_page::Docs>();
    }
    let Some(doc) = docs.0.get_mut(&page) else { return };
    // each <video>'s frame at its source time (data-source-time, the scene's frame's), as BevyPage::sync
    if let Err(e) = decoded.place(doc, nodes, videos) {
        warn_once!("raea-runtime-react's page: a video didn't decode: {e}");
    }
}

pub(crate) fn drop_doc(world: &mut World, page: Entity) {
    if let Some(mut t) = world.get_resource_mut::<raea_runtime_twins::bridge::TwinOps>() {
        t.0.remove(&page);
    }
    world.non_send_mut::<raea_sys_page::Docs>().0.remove(&page);
    world.despawn(page);
}

/// Which page image goes into which screen's image this frame: each <Screen>'s, once raea-render has made the screen.
pub(crate) fn screen_copies(world: &mut World) {
    let screens = raea_render::engine::screen_images(world);
    let mut copies = Vec::new();
    if let Some(wp) = world.get_resource::<WorldPage>() {
        for (name, s) in &wp.screens {
            let (Some(img), Some((_, dst, _, _))) = (world.get::<raea_sys_page::PageImage>(s.page), screens.iter().find(|x| &x.0 == name)) else { continue };
            copies.push((img.0.id(), dst.id()));
        }
    }
    world.resource_mut::<ScreenCopies>().0 = copies;
}

pub(crate) fn extract_copies(mut commands: Commands, copies: bevy::render::Extract<Res<ScreenCopies>>) {
    commands.insert_resource(copies.clone());
}

/// Each screen's page, painted this frame (raea-page's draw, in PrepareResources), copied into its screen's image
/// before the 3D draws: both premultiplied sRGB, as screen.wesl reads a picture.
pub(crate) fn copy_screens(copies: Option<Res<ScreenCopies>>, gpu: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>, device: Res<bevy::render::renderer::RenderDevice>, queue: Res<bevy::render::renderer::RenderQueue>) {
    let Some(copies) = copies.filter(|c| !c.0.is_empty()) else { return };
    let mut enc = device.wgpu_device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("screen pages") });
    for (src, dst) in &copies.0 {
        let (Some(s), Some(d)) = (gpu.get(*src), gpu.get(*dst)) else { continue };
        let size = wgpu::Extent3d { width: s.texture.width().min(d.texture.width()), height: s.texture.height().min(d.texture.height()), depth_or_array_layers: 1 };
        enc.copy_texture_to_texture(s.texture.as_image_copy(), d.texture.as_image_copy(), size);
    }
    queue.submit([enc.finish()]);
}

/// Whether two cameras draw to the same target (RenderTarget has no `==`).
pub(crate) fn same_target(a: &bevy::camera::RenderTarget, b: &bevy::camera::RenderTarget) -> bool {
    use bevy::camera::RenderTarget as T;
    match (a, b) {
        (T::Image(a), T::Image(b)) => a.handle == b.handle,
        (T::Window(a), T::Window(b)) => matches!((a, b), (bevy::window::WindowRef::Primary, bevy::window::WindowRef::Primary)) || matches!((a, b), (bevy::window::WindowRef::Entity(x), bevy::window::WindowRef::Entity(y)) if x == y),
        (T::TextureView(a), T::TextureView(b)) => a == b,
        _ => false,
    }
}

/// The page's image on the scene's camera, once both are there: a UI node over its whole target, in Bevy's picking.
/// With a layered Canvas3D (its 3D between the page's elements) the scene's camera draws into an image of the frame's
/// size instead, which the slot element shows (raea-page's ElementImages, GPU to GPU), and the page goes over the
/// target on a camera of its own. The page is drawn before the cameras, so the slot shows the 3D of the frame before.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn world_page_show(
    mut wp: ResMut<WorldPage>,
    target: Option<Res<raea_render::engine::SceneTarget>>,
    mut cameras: Query<(Entity, &mut bevy::camera::RenderTarget, &mut Camera), With<Camera3d>>,
    mut images: ResMut<Assets<Image>>,
    mut shows: ResMut<raea_sys_page::view3d::ElementImages>,
    textures: Res<PageTextures>,
    mut sized: ResMut<raea_sys_page::view3d::SizedImages>,
    mut last_slot: Local<Option<raea_runtime_twins::bridge::dom::NodeId>>,
    mut shown: Local<bool>,
    mut commands: Commands,
) {
    // the texture elements: each shows its texture while both are there
    let page = wp.page;
    let want: HashMap<_, _> = wp.texture_slots.iter().filter_map(|(tex, id)| Some(((page, raea_sys_page::ops::node(&wp.nodes, *id)?), textures.0.get(tex)?.clone()))).collect();
    let slot = wp.slot.and_then(|id| raea_sys_page::ops::node(&wp.nodes, id));
    shows.0.retain(|k, _| k.0 != page || want.contains_key(k) || slot.is_some_and(|n| k.1 == n));
    shows.0.extend(want);
    // the slot's image follows the slot's box (H7's SizedImages): the frame for a Canvas3D over the whole page, an
    // element's box for one inside a positioned element, so its 3D is drawn at that box's size and aspect
    if *last_slot != slot {
        if let Some(old) = last_slot.take() {
            sized.0.remove(&(page, old));
        }
        if let Some(n) = slot {
            sized.0.insert((page, n));
        }
        *last_slot = slot;
    }
    let Some(target) = target else { return };
    if let Some(n) = slot {
        let (w, h) = target.size;
        let image = wp.slot_image.get_or_insert_with(|| raea_sys_image::offscreen::target(&mut images, w, h)).clone();
        for (_, mut rt, mut cam) in cameras.iter_mut().filter(|(_, rt, _)| same_target(rt, &target.target)) {
            *rt = bevy::camera::RenderTarget::Image(image.clone().into());
            // the page under the 3D is seen where nothing is drawn
            cam.clear_color = ClearColorConfig::Custom(Color::NONE);
        }
        shows.0.insert((wp.page, n), image);
    }
    if *shown {
        return;
    }
    let cam = if slot.is_some() {
        commands.spawn((Camera2d, Camera { order: 10, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }, target.target.clone())).id()
    } else {
        let Some(cam) = cameras.iter().find(|(_, rt, _)| same_target(rt, &target.target)).map(|(e, _, _)| e) else { return };
        cam
    };
    commands.entity(cam).insert(raea_runtime_twins::overlay::PageOverlay { page: wp.page });
    // (the picture is the overlay pass's; the node is where picking finds the page)
    let node = Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() };
    commands.spawn((node, UiTargetCamera(cam), raea_runtime_input::pick::PageView::new(wp.page), bevy::picking::Pickable::IGNORE));
    *shown = true;
}
