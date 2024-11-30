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

use redis::aio::PubSubSink;
use std::{collections::BTreeMap, sync::OnceLock};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

use crate::pubsub::Event;

pub static FANOUT: OnceLock<Mutex<Fanout>> = OnceLock::new();

pub struct Fanout {
    /// A btreemap with `channel_id` or `guild_id` as the key and a vec of
    /// tokio senders as the value.
    pub channels: BTreeMap<String, Vec<UnboundedSender<Event>>>,
    pub sink: Option<PubSubSink>,
}

impl Default for Fanout {
    fn default() -> Self {
        Self::new()
    }
}

impl Fanout {
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
            sink: None,
        }
    }

    pub async fn add_sender(&mut self, channel: String, sender: UnboundedSender<Event>) {
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

    pub fn remove_sender(&mut self, channel: String, sender: UnboundedSender<Event>) {
        let senders = self.channels.entry(channel).or_default();

        senders.remove(
            senders
                .iter()
                .position(|s| s.same_channel(&sender))
                .unwrap(),
        );
    }
}
