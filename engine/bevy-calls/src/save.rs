//! moonshine-save (the declaration's `"calls": "save"`): saving marked entities to a file and loading them back
//! (docs/plugins.feature PL16). Its markers (Save, Unload) aren't reflected and saving is an event a Rust system
//! triggers, so TypeScript marks and saves through methods. moonshine-save's own observers do the work; these record
//! what each save and load did.
//!
//!   save/mark  {entities: [entity, ...], unload?}  -> null: marks them Save (saved, and replaced on load), or with
//!              unload Unload (never saved, despawned on load: a saved entity's visuals)
//!   save/save  {path}  -> {saved: [entity, ...]}: every Save entity's reflected, serializable components, as RON
//!   save/load  {path}  -> {loaded: [entity, ...]}: despawns the Save and Unload entities, then spawns the file's and
//!              marks them Save again (Save itself isn't saved, being unreflected)

use bevy::prelude::*;
use bevy_remote::BrpResult;
use moonshine_save::prelude::{Loaded, Save, SaveWorld, Saved, TriggerLoad, TriggerSave, Unload, LoadWorld, load_on_default_event, save_on_default_event};
use serde_json::{Value, json};

use crate::{add_method, entity, fail, params};

pub fn calls(app: &mut App) {
    app.add_observer(save_on_default_event).add_observer(load_on_default_event).add_observer(saved).add_observer(loaded);
    add_method(app, "save/mark", mark);
    add_method(app, "save/save", file(true));
    add_method(app, "save/load", file(false));
}

/// What the last save or load did: None until it finishes (moonshine-save logs why one failed)
#[derive(Resource, Default)]
struct Done(Option<Vec<Entity>>);

fn saved(s: On<Saved>, mut commands: Commands) {
    commands.insert_resource(Done(Some(s.entities().collect())));
}

fn loaded(l: On<Loaded>, mut commands: Commands) {
    let es: Vec<Entity> = l.entities().collect();
    for &e in &es {
        if let Ok(mut c) = commands.get_entity(e) {
            c.insert(Save);
        }
    }
    commands.insert_resource(Done(Some(es)));
}

#[derive(serde::Deserialize)]
struct MarkParams {
    entities: Vec<u64>,
    #[serde(default)]
    unload: bool,
}

fn mark(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: MarkParams = params(p)?;
    for bits in p.entities {
        let e = entity(bits)?;
        let mut at = world.get_entity_mut(e).map_err(|_| fail(format!("no entity {e}")))?;
        if p.unload { at.insert(Unload) } else { at.insert(Save) };
    }
    Ok(Value::Null)
}

#[derive(serde::Deserialize)]
struct PathParams {
    path: String,
}

/// save/save or save/load: one file through moonshine-save, and the entities it did, or why it didn't
fn file(save: bool) -> impl FnMut(In<Option<Value>>, &mut World) -> BrpResult {
    move |In(p), world| {
        let p: PathParams = params(p)?;
        world.insert_resource(Done(None));
        if save { world.trigger_save(SaveWorld::default_into_file(&p.path)) } else { world.trigger_load(LoadWorld::default_from_file(&p.path)) }
        world.flush();
        let (what, key) = if save { ("save to", "saved") } else { ("load from", "loaded") };
        let done = world.remove_resource::<Done>().and_then(|d| d.0).ok_or_else(|| fail(format!("{what} {} failed (the log says why)", p.path)))?;
        let bits: Vec<u64> = done.into_iter().map(Entity::to_bits).collect();
        Ok(Value::Object([(key.to_string(), json!(bits))].into_iter().collect()))
    }
}
