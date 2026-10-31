//! A composition's commits applied to the World, and the error its boundary rendered.

use super::*;

/// A composition's thrown error, as its error boundary rendered it (data-raea-error), seen as its commit was applied:
/// settle reads it without scanning every entity.
pub(crate) static COMPOSITION_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub(crate) fn apply_commits(
    mut commands: Commands,
    inbox: Res<Inbox>,
    mut ents: ResMut<ReactEntities>,
    mut props: Query<&mut ReactProps>,
    mut texts: Query<&mut ReactText>,
    mut tap: Option<ResMut<CommitTap>>,
    mut one: Option<ResMut<OneBatch>>,
) {
    let t_apply = Instant::now();
    let _count = ApplyTimer(t_apply, raea_core_alloc::here());
    let batches: Vec<Batch> = match one.as_mut() {
        Some(q) => {
            q.0.extend(inbox.0.lock().unwrap().try_iter());
            q.0.pop_front().into_iter().collect()
        }
        None => inbox.0.lock().unwrap().try_iter().collect(),
    };
    // props and text of entities spawned in this pass: commands are deferred, so an entity created here doesn't exist
    // for the queries until the pass ends, and an update to it in a later batch of the same pass was lost. A baked
    // seek feeds the keyframe (every node created) and the frames after it at once, and its videos came out a frame
    // behind (they read their source time from these props); the page mirror, applied directly, was right
    let mut fresh_props: HashMap<Entity, ReactProps> = HashMap::new();
    let mut fresh_text: HashMap<Entity, ReactText> = HashMap::new();
    for Batch(json) in batches {
        if let Some(t) = tap.as_mut() {
            t.0.push(json.clone());
        }
        // its updates read where they lie in the JSON, every other op parsed whole (raea-core-wire's raw: ZeroHeap Z4)
        let Some(ops) = raea_core_wire::raw::ops(&json) else {
            warn!("raea-runtime-react: a commit that isn't an op array");
            continue;
        };
        ents.commits += 1;
        for opx in &ops {
            ents.ops += 1;
            let op = match opx {
                raea_core_wire::raw::Op::Full(v) => v,
                raea_core_wire::raw::Op::Update(u) => {
                    let mut error = None;
                    raea_core_wire::raw::each_entry(u.props, |k, v| {
                        if k == "data-raea-error" {
                            error = serde_json::from_str::<String>(v.get()).ok();
                        }
                    });
                    if let Some(e) = error {
                        *COMPOSITION_ERROR.lock().unwrap() = Some(e);
                    }
                    if let Some(&e) = ents.by_id.get(&u.id) {
                        if let Some(p) = fresh_props.get_mut(&e) {
                            raea_core_wire::raw::merge_update(&mut p.0, u);
                        } else if let Ok(mut p) = props.get_mut(e) {
                            raea_core_wire::raw::merge_update(&mut p.0, u);
                        }
                    }
                    continue;
                }
            };
            if let Some(e) = op["props"].get("data-raea-error").and_then(|v| v.as_str()) {
                *COMPOSITION_ERROR.lock().unwrap() = Some(e.to_string());
            }
            let id = |k: &str| op[k].as_u64();
            match op["op"].as_str().unwrap_or("") {
                "reset" => {
                    let root = ents.by_id[&ROOT_ID];
                    for (i, e) in ents.by_id.drain() {
                        if i != ROOT_ID {
                            commands.entity(e).despawn();
                        }
                    }
                    ents.by_id.insert(ROOT_ID, root);
                }
                "create" => {
                    let mut p = ReactProps::default();
                    raea_core_wire::raw::merge(&mut p.0, &op["props"], None);
                    let e = commands.spawn((ReactNode(id("id").unwrap_or(0)), ReactKind(op["kind"].as_str().unwrap_or("").into()))).id();
                    fresh_props.insert(e, p);
                    if let Some(t) = op["text"].as_str() {
                        fresh_text.insert(e, ReactText(t.into()));
                    }
                    ents.by_id.insert(id("id").unwrap_or(0), e);
                }
                "createText" | "createTextSpan" => {
                    let kind = if op["op"] == "createText" { "#text" } else { "#span" };
                    let e = commands.spawn((ReactNode(id("id").unwrap_or(0)), ReactKind(kind.into()))).id();
                    fresh_text.insert(e, ReactText(op["text"].as_str().unwrap_or("").into()));
                    ents.by_id.insert(id("id").unwrap_or(0), e);
                }
                "append" => {
                    if let (Some(p), Some(c)) = (id("parent").and_then(|i| ents.by_id.get(&i)), id("child").and_then(|i| ents.by_id.get(&i))) {
                        commands.entity(*p).add_child(*c);
                    }
                }
                "insert" => {
                    if let (Some(&p), Some(&c), Some(&b)) = (
                        id("parent").and_then(|i| ents.by_id.get(&i)),
                        id("child").and_then(|i| ents.by_id.get(&i)),
                        id("before").and_then(|i| ents.by_id.get(&i)),
                    ) {
                        commands.queue(move |world: &mut World| {
                            let kids: Vec<Entity> = world.get::<Children>(p).map(|ks| ks.iter().filter(|e| *e != c).collect()).unwrap_or_default();
                            let at = kids.iter().position(|e| *e == b).unwrap_or(kids.len());
                            world.entity_mut(p).insert_children(at, &[c]);
                        });
                    }
                }
                "remove" => {
                    if let Some(c) = id("child").and_then(|i| ents.by_id.remove(&i)) {
                        // the subtree's ids go with it (React never puts a removed node back): only the child's was
                        // forgotten, and a live game's map of ids grew with every node it ever removed
                        commands.queue(move |world: &mut World| {
                            let mut stack = vec![c];
                            let mut gone = Vec::new();
                            while let Some(e) = stack.pop() {
                                if let Some(kids) = world.get::<Children>(e) {
                                    stack.extend(kids.iter());
                                }
                                if let Some(n) = world.get::<ReactNode>(e) {
                                    gone.push((n.0, e));
                                }
                            }
                            let mut ents = world.resource_mut::<ReactEntities>();
                            for (i, e) in gone {
                                if ents.by_id.get(&i) == Some(&e) {
                                    ents.by_id.remove(&i);
                                }
                            }
                            if let Ok(em) = world.get_entity_mut(c) {
                                em.despawn();
                            }
                        });
                    }
                }
                "update" => {
                    if let Some(&e) = id("id").and_then(|i| ents.by_id.get(&i)) {
                        let apply = |p: &mut ReactProps| {
                            raea_core_wire::raw::merge(&mut p.0, &op["props"], op.get("unset"));
                            // style keys React dropped (a transform going to nothing), as the DOM mirror does
                            if let (Some(style), Some(gone)) = (p.0.get_mut("style").and_then(|s| s.as_object_mut()), op["styleUnset"].as_array()) {
                                for k in gone.iter().filter_map(|k| k.as_str()) {
                                    style.remove(k);
                                }
                            }
                        };
                        if let Some(p) = fresh_props.get_mut(&e) {
                            apply(p);
                        } else if let Ok(mut p) = props.get_mut(e) {
                            apply(&mut p);
                        }
                    }
                }
                "updateText" => {
                    if let Some(&e) = id("id").and_then(|i| ents.by_id.get(&i)) {
                        let text = op["text"].as_str().unwrap_or("").to_string();
                        if let Some(t) = fresh_text.get_mut(&e) {
                            t.0 = text;
                        } else if let Ok(mut t) = texts.get_mut(e) {
                            t.0 = text;
                        }
                    }
                }
                other => warn!("raea-runtime-react: unknown op {other:?}"),
            }
        }
    }
    // this pass's new entities get their props and text as the last ops left them (an entity removed again in the same
    // pass is gone: try_insert skips it)
    for (e, p) in fresh_props {
        commands.entity(e).try_insert(p);
    }
    for (e, t) in fresh_text {
        commands.entity(e).try_insert(t);
    }
}

/// One pass of apply_commits timed and its allocations counted, into APPLY_NS and APPLY_ALLOC* as it drops.
pub(crate) struct ApplyTimer(pub(crate) Instant, pub(crate) (u64, u64));
impl Drop for ApplyTimer {
    fn drop(&mut self) {
        APPLY_NS.fetch_add(self.0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        let (n, b) = raea_core_alloc::here();
        APPLY_ALLOCS.fetch_add(n - self.1.0, std::sync::atomic::Ordering::Relaxed);
        APPLY_ALLOC_BYTES.fetch_add(b - self.1.1, std::sync::atomic::Ordering::Relaxed);
    }
}
