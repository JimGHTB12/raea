//! bevy_http_client (the declaration's `"calls": "http"`): a request from TypeScript and its response. The plugin
//! fetches on Bevy's IO task pool and hands the answer back in a later frame, so a request is an entity: the program
//! steps the App until its response is there.
//!
//!   http/request   {url, method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD", headers?: {name: value},
//!                   json?: any, form?: "a=1&b=2"}                    -> id (the request's entity)
//!   http/response  {id}  -> null while it is in flight, else {ok, status, status_text, url, headers: {name: value},
//!                           text} or {error}; the request's entity is despawned once read

use bevy::prelude::*;
use bevy_http_client::prelude::*;
use bevy_remote::BrpResult;
use serde_json::{Map, Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "http/request" => request, "http/response" => response);
    app.add_observer(answered).add_observer(failed);
}

/// The answer to a request made by http/request, on its entity
#[derive(Component)]
struct Answer(Value);

#[derive(serde::Deserialize)]
struct Request {
    url: String,
    method: Option<String>,
    #[serde(default)]
    headers: Map<String, Value>,
    json: Option<Value>,
    form: Option<String>,
}

#[derive(serde::Deserialize)]
struct Of {
    id: u64,
}

fn request(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let r: Request = params(p)?;
    let e = world.spawn_empty().id();
    let c = HttpClient::new_with_entity(e);
    let c = match r.method.as_deref().unwrap_or("GET").to_ascii_uppercase().as_str() {
        "GET" => c.get(&r.url),
        "POST" => c.post(&r.url),
        "PUT" => c.put(&r.url),
        "PATCH" => c.patch(&r.url),
        "DELETE" => c.delete(&r.url),
        "HEAD" => c.head(&r.url),
        m => {
            world.despawn(e);
            return Err(fail(format!("http method {m}: GET, POST, PUT, PATCH, DELETE or HEAD")));
        }
    };
    let pairs: Vec<(String, String)> = r.headers.iter().map(|(k, v)| (k.clone(), v.as_str().map_or_else(|| v.to_string(), str::to_string))).collect();
    let mut refs: Vec<(&str, &str)> = Vec::with_capacity(pairs.len());
    for (k, v) in &pairs {
        refs.push((k, v));
    }
    let c = if refs.is_empty() { c } else { c.headers(&refs) };
    let c = match (&r.json, &r.form) {
        (Some(j), _) => c.json(j),
        (None, Some(f)) => c.form_encoded(f),
        (None, None) => c,
    };
    match c.try_build() {
        Ok(req) => {
            world.write_message(req);
            Ok(json!(e.to_bits()))
        }
        Err(err) => {
            world.despawn(e);
            Err(fail(format!("http/request: {err}")))
        }
    }
}

fn answered(t: On<HttpObserved<HttpResponse>>, mut commands: Commands) {
    let r = &t.event().event;
    let headers: Map<String, Value> = r.headers.headers.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
    let a = json!({ "ok": r.ok, "status": r.status, "status_text": r.status_text, "url": r.url, "headers": headers, "text": r.text().unwrap_or_default() });
    commands.entity(t.event().entity).insert(Answer(a));
}

fn failed(t: On<HttpObserved<HttpResponseError>>, mut commands: Commands) {
    commands.entity(t.event().entity).insert(Answer(json!({ "error": t.event().event.err })));
}

fn response(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { id } = params(p)?;
    let e = Entity::try_from_bits(id).ok_or_else(|| fail(format!("{id} isn't a request")))?;
    let Ok(mut ent) = world.get_entity_mut(e) else { return Err(fail(format!("no request {id} (each response is read once)"))) };
    let Some(a) = ent.take::<Answer>() else { return Ok(Value::Null) };
    ent.despawn();
    Ok(a.0)
}
