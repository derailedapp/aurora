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

//! A simple temporary system for message passing feeding from Redis to our WebSockets.

use futures_util::StreamExt;
use redis::aio::{PubSub, PubSubSink};
use std::collections::BTreeMap;
use tokio::sync::mpsc::Sender;

use crate::pubsub::Event;

pub struct Fanout {
    /// A btreemap with `channel_id` or `guild_id` as the key and a vec of
    /// tokio senders as the value.
    pub channels: BTreeMap<String, Vec<Sender<Event>>>,
    pub sink: Option<PubSubSink>,
}

impl Default for Fanout {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Fanout {
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
            sink: None,
        }
    }

    pub async fn process_continuously<'b>(&'b mut self, sub: PubSub) {
        let (mut sink, mut stream) = sub.split();

        for channel in self.channels.keys() {
            sink.subscribe(channel.clone()).await.unwrap();
        }

        self.sink = Some(sink);

        while let Some(msg) = stream.next().await {
            let json = msg.get_payload_bytes();
            let model: Event = serde_json::from_slice(json).unwrap();

            let c = self.channels.get(msg.get_channel_name());

            if let Some(senders) = c {
                for sender in senders.iter() {
                    sender.send(model.clone()).await.unwrap();
                }
            }
        }
    }

    pub async fn add_sender<'b>(&'b mut self, channel: String, sender: Sender<Event>) {
        let senders = self.channels.entry(channel.clone()).or_default();

        if Vec::is_empty(senders) {
            // NOTE: should never be None since process_continuously
            // starts before the Gateway does.
            if let Some(mut sink) = self.sink.clone() {
                sink.subscribe(&channel).await.unwrap();
            }
        }

        senders.push(sender);
    }

    pub fn remove_sender(&mut self, channel: String, sender: Sender<Event>) {
        let senders = self.channels.entry(channel).or_default();

        senders.remove(
            senders
                .iter()
                .position(|s| s.same_channel(&sender))
                .unwrap(),
        );
    }
}
