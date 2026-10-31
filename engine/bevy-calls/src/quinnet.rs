//! bevy_quinnet (the declaration's `"calls": "quinnet"`): a QUIC server and client from TypeScript. QuinnetServer and
//! QuinnetClient are resources with no reflected API, so a program opens, links and messages through these. One App
//! can hold both halves (the catalog's proof listens and connects to itself); payloads are text on channel 0 unless a
//! channel is given, and stay buffered until received.
//!
//!   quinnet/listen   {port?}                           -> {port}; a self-signed certificate, bound to 127.0.0.1
//!                                                         (port 0 or none: any free one)
//!   quinnet/connect  {port, host?}                     -> the connection's id; skips certificate verification
//!   quinnet/status                                     -> {listening, clients: [id], client: "Connecting" |
//!                                                         "Connected" | "Disconnected" | null, id: the server's id
//!                                                         for this client or null}
//!   quinnet/send     {text, client?: id, channel?}     -> null; with a client, the server sends to it, else the
//!                                                         client sends to the server
//!   quinnet/receive  {client?: id, channel?}           -> [text]; with a client, what the server got from it,
//!                                                         else what the client got from the server

use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::client::certificate::CertificateVerificationMode;
use bevy_quinnet::client::connection::{ClientAddrConfiguration, ConnectionState};
use bevy_quinnet::client::{ClientConnectionConfiguration, QuinnetClient};
use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use bevy_quinnet::server::{EndpointAddrConfiguration, QuinnetServer, ServerEndpointConfiguration};
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::{fail, params};

const LOCAL: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

pub fn calls(app: &mut App) {
    crate::methods!(app,
        "quinnet/listen" => listen, "quinnet/connect" => connect, "quinnet/status" => status,
        "quinnet/send" => send, "quinnet/receive" => receive,
    );
}

#[derive(serde::Deserialize, Default)]
struct Listen {
    port: Option<u16>,
}

#[derive(serde::Deserialize)]
struct Connect {
    port: u16,
    host: Option<IpAddr>,
}

#[derive(serde::Deserialize)]
struct Send {
    text: String,
    client: Option<u64>,
    channel: Option<u8>,
}

#[derive(serde::Deserialize, Default)]
struct Receive {
    client: Option<u64>,
    channel: Option<u8>,
}

fn listen(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Listen { port } = if p.is_some() { params(p)? } else { Listen::default() };
    let config = ServerEndpointConfiguration {
        addr_config: EndpointAddrConfiguration::from_ip(LOCAL, port.unwrap_or(0)),
        cert_mode: CertificateRetrievalMode::GenerateSelfSigned { server_hostname: LOCAL.to_string() },
        defaultables: default(),
    };
    let mut server = world.resource_mut::<QuinnetServer>();
    server.start_endpoint(config).map_err(|e| fail(format!("quinnet/listen: {e}")))?;
    Ok(json!({ "port": server.endpoint().local_addr().port() }))
}

fn connect(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let c: Connect = params(p)?;
    let config = ClientConnectionConfiguration {
        addr_config: ClientAddrConfiguration::from_ips(c.host.unwrap_or(LOCAL), c.port, LOCAL, 0),
        cert_mode: CertificateVerificationMode::SkipVerification,
        defaultables: default(),
    };
    let id = world.resource_mut::<QuinnetClient>().open_connection(config).map_err(|e| fail(format!("quinnet/connect: {e}")))?;
    Ok(json!(id))
}

fn status(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let server = world.resource::<QuinnetServer>();
    let clients = server.get_endpoint().map(|e| e.clients()).unwrap_or_default();
    let connection = world.resource::<QuinnetClient>().get_connection();
    let state = connection.map(|c| match c.state() {
        ConnectionState::Connecting => "Connecting",
        ConnectionState::Connected => "Connected",
        ConnectionState::Disconnected => "Disconnected",
    });
    Ok(json!({
        "listening": server.is_listening(),
        "clients": clients,
        "client": state,
        "id": connection.and_then(|c| c.client_id()),
    }))
}

fn send(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let s: Send = params(p)?;
    let channel = s.channel.unwrap_or(0);
    let sent = match s.client {
        Some(client) => world
            .resource_mut::<QuinnetServer>()
            .get_endpoint_mut()
            .ok_or_else(|| fail("quinnet/send: no server (quinnet/listen opens one)"))?
            .send_payload_on(client, channel, s.text)
            .map_err(|e| e.to_string()),
        None => world
            .resource_mut::<QuinnetClient>()
            .get_connection_mut()
            .ok_or_else(|| fail("quinnet/send: no connection (quinnet/connect opens one)"))?
            .send_payload_on(channel, s.text)
            .map_err(|e| e.to_string()),
    };
    sent.map_err(|e| fail(format!("quinnet/send: {e}")))?;
    Ok(Value::Null)
}

fn receive(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let r: Receive = if p.is_some() { params(p)? } else { Receive::default() };
    let channel = r.channel.unwrap_or(0);
    let mut got = Vec::new();
    match r.client {
        Some(client) => {
            let mut server = world.resource_mut::<QuinnetServer>();
            let Some(endpoint) = server.get_endpoint_mut() else { return Err(fail("quinnet/receive: no server")) };
            while let Some(b) = endpoint.receive_payload(client, channel).map_err(|e| fail(format!("quinnet/receive: {e}")))? {
                got.push(String::from_utf8_lossy(&b).into_owned());
            }
        }
        None => {
            let mut client = world.resource_mut::<QuinnetClient>();
            let Some(connection) = client.get_connection_mut() else { return Err(fail("quinnet/receive: no connection")) };
            while let Some(b) = connection.try_receive_payload(channel) {
                got.push(String::from_utf8_lossy(&b).into_owned());
            }
        }
    }
    Ok(json!(got))
}
