// Copyright (C) 2024 V.J. De Chico
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Temporary Redis-based implementation of the Derailed Gateway

use aurora_db::{account::Account, actor::Actor, channel::Channel, guild::Guild};
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::{
    error::{ErrorMessage, OVTError},
    fanout::{Fanout, FANOUT},
    pubsub::Event,
    state::OVTState,
    token::get_user,
};

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "t", content = "d")]
pub enum GatewayMessage {
    Identify {
        token: String,
    },
    Ready {
        // NOTE: fields are boxed to avoid the enum
        // from growing too large
        account: Box<Account>,
        actor: Box<Actor>,
        guilds: Vec<Guild>,
        guild_channels: Vec<Channel>,
    },
}

pub struct GatewayConnection {
    actor: Option<Actor>,
    account: Option<Account>,
    sink: tokio::sync::mpsc::UnboundedSender<Event>,
    subscriptions: Vec<String>,
}

pub async fn handle_ws_request(
    ws: WebSocketUpgrade,
    State(state): State<OVTState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

async fn handle_connection(sock: WebSocket, mut state: OVTState) {
    let (mut ws_sink, mut ws_stream) = sock.split();
    let (sink, mut stream) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let (event_sink, mut event_stream) = tokio::sync::mpsc::unbounded_channel();
    let conn = Mutex::new(GatewayConnection::new(event_sink));

    let user_sink = sink.clone();

    tokio::spawn(async move {
        while let Some(msg) = stream.recv().await {
            let model = serde_json::from_slice::<GatewayMessage>(&msg.into_data());

            if let Ok(message) = model {
                let mut c = conn.lock().await;
                let res = c.handle_message(message, &mut state).await;
                if let Ok(msg) = res {
                    let json = serde_json::to_string(&msg).unwrap();
                    let _ = user_sink.send(axum::extract::ws::Message::Text(json));
                } else {
                    let err = res.err().unwrap();
                    let json = serde_json::to_string(&err).unwrap();
                    let _ = user_sink.send(axum::extract::ws::Message::Close(Some(CloseFrame {
                        code: 4000,
                        reason: json.into(),
                    })));
                    stream.close();
                    return;
                }
            } else {
                let _ = user_sink.send(axum::extract::ws::Message::Close(Some(CloseFrame {
                    code: 4000,
                    reason: "Invalid JSON message".into(),
                })));
                stream.close();
                break;
            }
        }
    });

    let event_ws_sink = sink.clone();
    tokio::spawn(async move {
        while let Some(event) = event_stream.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            let _ = event_ws_sink.send(axum::extract::ws::Message::Text(json));
        }
    });

    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_stream.next().await {
            let _ = ws_sink.send(msg).await;
        }
    });
}

impl GatewayConnection {
    pub fn new(sink: UnboundedSender<Event>) -> Self {
        Self {
            account: None,
            actor: None,
            subscriptions: Vec::new(),
            sink,
        }
    }

    pub async fn handle_message(
        &mut self,
        msg: GatewayMessage,
        state: &mut OVTState,
    ) -> Result<GatewayMessage, ErrorMessage> {
        match msg {
            GatewayMessage::Identify { token } => {
                let mut m = HeaderMap::new();
                m.insert("authorization", HeaderValue::from_str(&token).unwrap());
                let (actor, account) = get_user(&m, &state.key, &state.pg)
                    .await
                    .map_err(|(_, err)| err.0)?;

                self.actor = Some(actor.clone());
                self.account = Some(account.clone());

                let guilds = sqlx::query_as!(
                    Guild,
                    "SELECT * FROM guilds WHERE id IN (SELECT guild_id FROM guild_members WHERE user_id = $1);",
                    &actor.id
                ).fetch_all(&state.pg).await.map_err(|_| OVTError::InternalServerError.to_resp().1.0)?;

                let mut fo = FANOUT
                    .get_or_init(|| tokio::sync::Mutex::new(Fanout::new()))
                    .lock()
                    .await;
                for guild in guilds.iter() {
                    self.subscriptions.push(guild.id.clone());
                    fo.add_sender(guild.id.clone(), self.sink.clone()).await;
                }

                Ok(GatewayMessage::Ready {
                    guilds,
                    account: Box::new(account),
                    actor: Box::new(actor.clone()),
                    guild_channels: sqlx::query_as!(
                        Channel,
                        "SELECT * FROM channels WHERE guild_id IN (SELECT guild_id FROM guild_members WHERE user_id = $1);",
                        &actor.id
                    ).fetch_all(&state.pg).await.map_err(|_| OVTError::InternalServerError.to_resp().1.0)?,
                })
            }
            _ => Err(OVTError::ServerSentEvent.to_resp().1 .0),
        }
    }
}

impl Drop for GatewayConnection {
    fn drop(&mut self) {
        let mut fo = FANOUT
            .get_or_init(|| tokio::sync::Mutex::new(Fanout::new()))
            .blocking_lock();

        for sub in self.subscriptions.clone().into_iter() {
            fo.remove_sender(sub, self.sink.clone());
        }
    }
}
