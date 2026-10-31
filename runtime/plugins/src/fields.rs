//! A plugin's rows as reflected values: components and resources by type path, their fields read from a row's values
//! (assets by path, gizmos, pictures, entity references).

use super::*;

/// What a component row's Handle and Entity fields resolve against (the edit loaded last).
#[derive(Resource, Default)]
pub struct Refs {
    pub dir: PathBuf,
    pub names: BTreeMap<String, Entity>,
    pub media: Vec<Tup>,
}

/// The images cameras draw into, by the name a Handle<Image> field gives them ("render:NAME"), and their sizes.
#[derive(Resource, Default)]
pub struct RenderImages(pub BTreeMap<String, (Handle<Image>, (u32, u32))>);
/// Assets a handle field started loading by path, which the load waits for before the first frame (await_loads).
#[derive(Resource, Default)]
pub struct Loading(pub Vec<(String, UntypedHandle)>);
/// The live game's world-call natives (bevy.world.send, bevy.world.answer), for a host to register before a page loads.
pub fn register_world_natives() {
    raea_bevy_world::register_natives();
}

/// The live game's window settings (raea/window) and their version, for the host that owns the window.
pub fn window_settings() -> (u64, Option<Value>) {
    raea_bevy_world::window_settings()
}

/// A world call's entity named as the edit names it: a part's name or row id.
pub fn row_names(world: &mut World, v: &Value) -> Result<Option<Entity>, String> {
    let n = v.as_str().ok_or_else(|| format!("an entity is its bits, or a part's name or row id, not {v}"))?;
    Ok(world.get_resource::<Refs>().and_then(|r| r.names.get(n).copied()))
}

/// A picture as a Handle<Image> field gives it (a path in the edit, an inline {width, height, pixels}, "render:NAME"),
/// for a material built outside the component rows (raea-sys-material's shader_ext.rs: a surface shader's textures).
/// Called once the rows are applied (apply_rows), so a "render:NAME" meets the camera that draws into it.
pub fn image(world: &mut World, v: &Value) -> Result<Handle<Image>, String> {
    let refs = world.remove_resource::<Refs>().unwrap_or_default();
    let reg = world.resource::<AppTypeRegistry>().clone();
    let got = {
        let reg = reg.read();
        let mut f = Fields { world, refs: &refs };
        f.handle(&reg, TypeId::of::<Image>(), v).map(|h| h.typed::<Image>())
    };
    world.insert_resource(refs);
    got
}

/// Bevy's reflect deserializer, taught the two kinds of field that aren't data (bevy_reflect's
/// ReflectDeserializerProcessor, the hook it has for this):
///   a Handle<A>  "path/in/the/edit.png" or "model.glb#Mesh0/Primitive0": loaded through the AssetServer
///                {fields}: an inline A, deserialized the same way (its own handle fields too) over A's default and
///                added to Assets<A> (ReflectAsset::add)
///                "shape:box(1, 1, 1)": a Mesh, the Manifold expression built as a <Mesh>'s shape is
///   an Entity    a part's name (<Mesh name>, <Entity name>) or row id ("mesh0"), or entity bits (a number)
/// Any other type is given back to the deserializer untouched.
pub struct Fields<'w> {
    pub world: &'w mut World,
    pub refs: &'w Refs,
}

impl Fields<'_> {
    /// Triangles as a Mesh in Assets<Mesh> (its texture coordinates given, or projected onto its bounds).
    pub fn mesh(&mut self, tris: &raea_core_shape::Tris, uvs: Option<Vec<[f32; 2]>>) -> UntypedHandle {
        let mut m = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.unwrap_or_else(|| super::mesh::box_uvs(&tris.positions, &tris.normals)));
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, tris.positions.clone());
        m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, tris.normals.clone());
        self.world.resource_mut::<Assets<Mesh>>().add(m).untyped()
    }

    pub fn entity(&self, v: &Value) -> Result<Entity, String> {
        match v {
            Value::String(n) => self.refs.names.get(n).copied().or_else(|| {
                let mut q = self.world.try_query::<(Entity, &Name)>()?;
                q.iter(self.world).find(|(_, m)| m.as_str() == n).map(|(e, _)| e)
            }).ok_or_else(|| format!("an Entity field names {n:?}, and no part has that name or row id (names: {})", self.refs.names.keys().filter(|k| !k.contains('-')).cloned().collect::<Vec<_>>().join(", "))),
            Value::Number(b) => b.as_u64().and_then(|b| Entity::try_from_bits(b)).ok_or_else(|| format!("an Entity field's bits {b} aren't an entity")),
            _ => Err(format!("an Entity field is a part's name or row id, or entity bits, got {v}")),
        }
    }

    /// An asset path's file, found as a <Model src> is (next to the edit, or a media row), any #label kept, loading.
    fn load_path(&mut self, asset: TypeId, name: &str, x: &str) -> Result<UntypedHandle, String> {
        let (file, label) = match x.split_once('#') {
            Some((f, l)) => (f, Some(l)),
            None => (x, None),
        };
        let found = resolve_src(&self.refs.media, file, &self.refs.dir)?;
        if !found.exists() {
            return Err(format!("a Handle<{name}> names {file:?}, and there's no such file ({})", found.display()));
        }
        let path = match label {
            Some(l) => format!("{}#{l}", found.to_string_lossy()),
            None => found.to_string_lossy().to_string(),
        };
        let h = self.world.resource::<AssetServer>().load_builder().load_erased(asset, path.clone());
        self.world.get_resource_or_init::<Loading>().0.push((path, h.clone()));
        Ok(h)
    }

    pub fn handle(&mut self, reg: &TypeRegistry, asset: TypeId, v: &Value) -> Result<UntypedHandle, String> {
        // a GizmoAsset (a retained <Gizmo>'s lines) isn't reflected, so its buffer's own fields are read here
        if asset == TypeId::of::<bevy::gizmos::GizmoAsset>() {
            let g = gizmo_asset(v)?;
            return Ok(self.world.resource_mut::<Assets<bevy::gizmos::GizmoAsset>>().add(g).untyped());
        }
        // a material written in TypeScript: its WGSL or WESL, uniforms and textures (shader_material.rs). Before the
        // registry's look: ShaderMaterial isn't Reflect (its bind group is its whole interface), so it has no
        // registration to find, and a MeshMaterial3d<ShaderMaterial> was refused as "an asset type that isn't registered"
        if asset == TypeId::of::<super::shader_material::ShaderMaterial>() {
            let reg2 = reg;
            let m = {
                let refs = self.refs;
                let image = |world: &mut World, v: &Value| -> Result<Handle<Image>, String> {
                    let mut f = Fields { world, refs };
                    f.handle(reg2, TypeId::of::<Image>(), v).map(|h| h.typed::<Image>())
                };
                super::shader_material::build(self.world, v, image)?
            };
            return Ok(self.world.resource_mut::<Assets<super::shader_material::ShaderMaterial>>().add(m).untyped());
        }
        // a Shader (bevy_smud's SDF and fill, say) isn't reflected either: {"Uuid": "…"} names one a plugin loaded as an
        // internal asset, and a path names a .wgsl or .wesl file, added as a shader
        if asset == TypeId::of::<bevy::shader::Shader>() {
            return match v {
                Value::Object(o) if o.len() == 1 && o.get("Uuid").is_some_and(Value::is_string) => {
                    let u = bevy::asset::uuid::Uuid::parse_str(o["Uuid"].as_str().unwrap()).map_err(|e| format!("a Handle<Shader>'s Uuid: {e}"))?;
                    Ok(Handle::<bevy::shader::Shader>::Uuid(u, std::marker::PhantomData).untyped())
                }
                Value::String(x) => {
                    // added under a module path of its own, not loaded: Bevy names a loaded WESL module after its
                    // file's path, and a path's dashes aren't WESL names (smud's `import package::…::bevy-ts::…`
                    // failed to parse)
                    let found = resolve_src(&self.refs.media, x, &self.refs.dir)?;
                    let source = std::fs::read_to_string(&found).map_err(|e| format!("a Handle<Shader> names {x:?}: {e} ({})", found.display()))?;
                    let stem: String = found.file_stem().unwrap_or_default().to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
                    let mut h = std::hash::DefaultHasher::new();
                    std::hash::Hash::hash(&found, &mut h);
                    let path = |ext: &str| format!("embedded://raea_runtime_plugins/shaders/s_{stem}_{:016x}.{ext}", std::hash::Hasher::finish(&h));
                    let shader = match found.extension().and_then(|e| e.to_str()) {
                        Some("wesl") => bevy::shader::Shader::from_wesl(source, path("wesl")),
                        Some("wgsl") => bevy::shader::Shader::from_wgsl(source, path("wgsl")),
                        _ => return Err(format!("a Handle<Shader> is a .wgsl or .wesl file, got {x:?}")),
                    };
                    Ok(self.world.resource_mut::<Assets<bevy::shader::Shader>>().add(shader).untyped())
                }
                _ => Err(format!("a Handle<Shader> is a .wgsl or .wesl path, or {{\"Uuid\": …}} for a plugin's own, got {v}")),
            };
        }
        // an asset type that isn't Reflect (flair's StyleSheet) has no registration, but its loader takes a path all
        // the same; only an inline one needs reflecting
        let Some(ar) = reg.get(asset) else {
            return match v {
                Value::String(x) => self.load_path(asset, "asset", x),
                _ => Err(format!("a handle to an asset type that isn't reflected is an asset path, got {v}")),
            };
        };
        let name = ar.type_info().type_path_table().short_path();
        match v {
            Value::String(x) if asset == TypeId::of::<Mesh>() && x.starts_with("shape:") => {
                let tris = raea_core_shape::cached(&x["shape:".len()..]).map_err(|e| format!("a Mesh handle's shape: {e}"))?;
                Ok(self.mesh(&tris, None))
            }
            // a Mesh's own vertex buffers ({positions, indices?, normals?, uvs?}): Mesh's reflection leaves its
            // attributes out, so this is read here
            Value::Object(o) if asset == TypeId::of::<Mesh>() && o.contains_key("positions") => {
                let (tris, uvs) = super::mesh::buffer_tris(&v.to_string()).map_err(|e| format!("a Mesh handle: {e}"))?;
                let colors = super::mesh::buffer_colors(&v.to_string()).map_err(|e| format!("a Mesh handle: {e}"))?;
                let h = self.mesh(&tris, uvs);
                if let Some(c) = colors {
                    if let Some(mut m) = self.world.resource_mut::<Assets<Mesh>>().get_mut(h.id().typed::<Mesh>()) {
                        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
                    }
                }
                Ok(h)
            }
            // a picture made in code: {width, height, pixels} with pixels r, g, b, a bytes (0 to 255) row by row from
            // the top, or one "#rrggbb"/"#rrggbbaa" a pixel; `linear: true` keeps them as data (a normal or roughness
            // map), sRGB otherwise; `nearest: true` samples without smoothing (pixel art)
            Value::Object(o) if asset == TypeId::of::<Image>() && o.contains_key("pixels") => {
                let img = pixel_image(o)?;
                Ok(self.world.resource_mut::<Assets<Image>>().add(img).untyped())
            }
            // one of Bevy's primitive shapes, its fields over its default ({primitive: "Torus", major_radius: 1})
            Value::Object(o) if asset == TypeId::of::<Mesh>() && o.contains_key("primitive") => {
                let m = super::mesh::primitive_mesh(o).map_err(|e| format!("a Mesh handle: {e}"))?;
                Ok(self.world.resource_mut::<Assets<Mesh>>().add(m).untyped())
            }
            // a named image a camera draws into ("render:mirror" or "render:mirror:256x256", 512 x 512 if not
            // given): every field naming it gets the same image, so a camera's RenderTarget and a material's texture
            // meet in it
            Value::String(x) if asset == TypeId::of::<Image>() && x.starts_with("render:") => {
                let rest = &x["render:".len()..];
                let (name, size) = match rest.rsplit_once(':') {
                    Some((n, sz)) if sz.contains('x') => (n, Some(sz)),
                    _ => (rest, None),
                };
                let (w, h) = match size {
                    Some(sz) => {
                        let (w, h) = sz.split_once('x').unwrap();
                        let p = |d: &str| d.parse::<u32>().ok().filter(|n| (1..=8192).contains(n));
                        p(w).zip(p(h)).ok_or_else(|| format!("{x:?}: a render image's size is WxH in pixels (1 to 8192)"))?
                    }
                    None => (512, 512),
                };
                if name.is_empty() {
                    return Err(format!("{x:?}: a render image needs a name (\"render:mirror\")"));
                }
                if let Some(hd) = self.world.get_resource::<RenderImages>().and_then(|r| r.0.get(name)) {
                    if size.is_some() && hd.1 != (w, h) {
                        return Err(format!("{x:?}: the render image {name:?} is already {}x{}", hd.1.0, hd.1.1));
                    }
                    return Ok(hd.0.clone().untyped());
                }
                let img = Image::new_target_texture(w, h, bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, None);
                let hd = self.world.resource_mut::<Assets<Image>>().add(img);
                self.world.get_resource_or_init::<RenderImages>().0.insert(name.to_string(), (hd.clone(), (w, h)));
                Ok(hd.untyped())
            }
            Value::String(x) => self.load_path(asset, name, x),
            Value::Object(_) => {
                let ra = ar.data::<ReflectAsset>().ok_or_else(|| format!("{name} can't be given inline (no ReflectAsset); give a path"))?;
                let given = TypedReflectDeserializer::with_processor(ar, reg, self).deserialize(v.clone()).map_err(|e| format!("an inline {name}: {e}"))?;
                let full: Box<dyn PartialReflect> = match ar.data::<ReflectDefault>() {
                    Some(d) => {
                        let mut x = d.default();
                        x.try_apply(given.as_partial_reflect()).map_err(|e| format!("an inline {name}: {e}"))?;
                        x.into_partial_reflect()
                    }
                    None => given,
                };
                if ar.data::<ReflectFromReflect>().and_then(|f| f.from_reflect(full.as_partial_reflect())).is_none() {
                    return Err(format!("an inline {name} has no default, so give every field"));
                }
                Ok(ra.add(self.world, full.as_partial_reflect()))
            }
            _ => Err(format!("a Handle<{name}> is an asset path, an inline {name} (an object){}, got {v}", if asset == TypeId::of::<Mesh>() { ", \"shape:<a Manifold expression>\" or {positions, indices, normals, uvs}" } else { "" })),
        }
    }
}

/// A GizmoAsset from its GizmoBuffer's fields: list_positions (segment ends, in pairs) and strip_positions (a line
/// through them; null or NaN breaks it), each [x, y, z], with list_colors and strip_colors one per position ("#rrggbb"
/// sRGB or [r, g, b, a] linear); `color` fills colours not given.
pub fn gizmo_asset(v: &Value) -> Result<bevy::gizmos::GizmoAsset, String> {
    let o = v.as_object().ok_or_else(|| format!("a GizmoAsset is {{list_positions, list_colors, strip_positions, strip_colors, color}}, got {v}"))?;
    let point = |p: &Value| -> Result<Vec3, String> {
        match p {
            Value::Null => Ok(Vec3::NAN),
            Value::Array(a) if a.len() == 3 => Ok(Vec3::new(
                a[0].as_f64().unwrap_or(f64::NAN) as f32, a[1].as_f64().unwrap_or(f64::NAN) as f32, a[2].as_f64().unwrap_or(f64::NAN) as f32)),
            _ => Err(format!("a gizmo position is [x, y, z] (or null to break a strip), got {p}")),
        }
    };
    let colour = |c: &Value| -> Result<LinearRgba, String> {
        match c {
            Value::String(s) => Srgba::hex(s.trim_start_matches('#')).map(LinearRgba::from).map_err(|e| format!("a gizmo colour {s:?}: {e}")),
            Value::Array(a) if a.len() >= 3 => {
                let f = |i: usize| a.get(i).and_then(Value::as_f64).unwrap_or(1.0) as f32;
                Ok(LinearRgba::new(f(0), f(1), f(2), f(3)))
            }
            _ => Err(format!("a gizmo colour is \"#rrggbb\" or [r, g, b, a], got {c}")),
        }
    };
    let fill = match o.get("color") { Some(c) => colour(c)?, None => LinearRgba::WHITE };
    let read = |pk: &str, ck: &str| -> Result<(Vec<Vec3>, Vec<LinearRgba>), String> {
        let ps = match o.get(pk) { Some(Value::Array(a)) => a.iter().map(point).collect::<Result<Vec<_>, _>>()?, None => vec![], Some(x) => return Err(format!("a GizmoAsset's {pk} is a list of [x, y, z], got {x}")) };
        let given = match o.get(ck) { Some(Value::Array(a)) => a.iter().map(colour).collect::<Result<Vec<_>, _>>()?, None => vec![], Some(x) => return Err(format!("a GizmoAsset's {ck} is a list of colours, got {x}")) };
        let cs = (0..ps.len()).map(|i| given.get(i).copied().unwrap_or(fill)).collect();
        Ok((ps, cs))
    };
    let (lp, lc) = read("list_positions", "list_colors")?;
    if lp.len() % 2 == 1 {
        return Err(format!("a GizmoAsset's list_positions are segment ends in pairs, got {}", lp.len()));
    }
    let (sp, sc) = read("strip_positions", "strip_colors")?;
    let mut g = bevy::gizmos::GizmoAsset::new();
    g.list_positions = lp;
    g.list_colors = lc;
    g.strip_positions = sp;
    g.strip_colors = sc;
    Ok(g)
}

/// An Image from its pixels (Fields::handle's procedural picture).
pub fn pixel_image(o: &serde_json::Map<String, Value>) -> Result<Image, String> {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let dim = |k: &str| o.get(k).and_then(Value::as_u64).filter(|n| (1..=8192).contains(n)).map(|n| n as u32).ok_or_else(|| format!("a picture's {k} is a whole number of pixels, 1 to 8192"));
    let (w, h) = (dim("width")?, dim("height")?);
    let px = o.get("pixels").and_then(Value::as_array).ok_or("a picture's pixels are a list")?;
    let n = (w * h) as usize;
    let data: Vec<u8> = if !px.is_empty() && px.iter().all(Value::is_string) {
        if px.len() != n {
            return Err(format!("a picture's pixels: one colour for each of its {w} x {h} pixels, got {}", px.len()));
        }
        let mut d = Vec::with_capacity(n * 4);
        for c in px {
            let s = c.as_str().unwrap();
            let c = Srgba::hex(s.trim_start_matches('#')).map_err(|e| format!("a picture's pixel {s:?}: {e}"))?;
            d.extend(c.to_u8_array());
        }
        d
    } else {
        if px.len() != n * 4 {
            return Err(format!("a picture's pixels: r, g, b, a for each of its {w} x {h} pixels ({} numbers), got {}", n * 4, px.len()));
        }
        px.iter().map(|x| x.as_u64().filter(|v| *v <= 255).map(|v| v as u8).ok_or_else(|| format!("a picture's pixel values are 0 to 255, got {x}"))).collect::<Result<_, _>>()?
    };
    let linear = o.get("linear").and_then(Value::as_bool).unwrap_or(false);
    let format = if linear { TextureFormat::Rgba8Unorm } else { TextureFormat::Rgba8UnormSrgb };
    let mut img = Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, data, format, bevy::asset::RenderAssetUsages::default());
    if o.get("nearest").and_then(Value::as_bool).unwrap_or(false) {
        img.sampler = bevy::image::ImageSampler::nearest();
    }
    Ok(img)
}

/// Whether a struct's own fields include an Entity.
pub fn entity_fields(ti: &TypeInfo) -> bool {
    match ti {
        TypeInfo::Struct(s) => s.iter().any(|f| f.type_id() == TypeId::of::<Entity>()),
        TypeInfo::TupleStruct(s) => s.iter().any(|f| f.type_id() == TypeId::of::<Entity>()),
        _ => false,
    }
}

impl Fields<'_> {
    /// A struct read field by field through the registry (a one-field tuple struct from its field's value alone).
    pub fn by_fields(&mut self, r: &TypeRegistration, reg: &TypeRegistry, v: &Value) -> Result<Box<dyn PartialReflect>, String> {
        let field = |me: &mut Self, type_id: TypeId, x: &Value| -> Result<Box<dyn PartialReflect>, String> {
            let fr = reg.get(type_id).ok_or("a field of a type that isn't registered")?;
            TypedReflectDeserializer::with_processor(fr, reg, me).deserialize(x.clone()).map_err(|e| e.to_string())
        };
        match r.type_info() {
            TypeInfo::TupleStruct(ts) => {
                let items: Vec<Value> = match (ts.field_len(), v) {
                    (1, x) => vec![x.clone()],
                    (_, Value::Array(a)) => a.clone(),
                    _ => return Err(format!("{} is [its fields], got {v}", ts.type_path())),
                };
                let mut out = bevy::reflect::tuple_struct::DynamicTupleStruct::default();
                out.set_represented_type(Some(r.type_info()));
                for (f, x) in ts.iter().zip(&items) {
                    out.insert_boxed(field(self, f.type_id(), x)?);
                }
                Ok(Box::new(out))
            }
            TypeInfo::Struct(st) => {
                let mut out = bevy::reflect::structs::DynamicStruct::default();
                out.set_represented_type(Some(r.type_info()));
                for f in st.iter() {
                    if let Some(x) = v.get(f.name()) {
                        out.insert_boxed(f.name(), field(self, f.type_id(), x)?);
                    }
                }
                Ok(Box::new(out))
            }
            _ => Err("not a struct".into()),
        }
    }
}

impl ReflectDeserializerProcessor for Fields<'_> {
    fn try_deserialize<'de, D>(&mut self, r: &TypeRegistration, reg: &TypeRegistry, d: D) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::Deserialize;
        use serde::de::Error;
        if r.type_id() == TypeId::of::<Entity>() {
            let v = Value::deserialize(d)?;
            return Ok(Ok(Box::new(self.entity(&v).map_err(D::Error::custom)?)));
        }
        // a type serde reads whole (ChildOf, a relationship's target) would read its Entity fields as bits: read it
        // field by field instead, so each field comes back through here
        if r.data::<ReflectDeserialize>().is_some() && entity_fields(r.type_info()) {
            let v = Value::deserialize(d)?;
            return self.by_fields(r, reg, &v).map(Ok).map_err(D::Error::custom);
        }
        // an Arc<str> (bevy_rich_text3d's font name) is reflected opaque without serde's data: it reads as a string
        if r.type_id() == TypeId::of::<bevy::platform::sync::Arc<str>>() {
            let s = String::deserialize(d)?;
            return Ok(Ok(Box::new(bevy::platform::sync::Arc::<str>::from(s))));
        }
        let Some(rh) = r.data::<ReflectHandle>() else { return Ok(Err(d)) };
        let v = Value::deserialize(d)?;
        let h = self.handle(reg, rh.asset_type_id(), &v).map_err(D::Error::custom)?;
        Ok(Ok(rh.typed(h).into_partial_reflect()))
    }
}
