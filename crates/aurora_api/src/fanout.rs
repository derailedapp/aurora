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

use std::collections::BTreeMap;
use tokio::sync::mpsc::Sender;

use crate::pubsub::Event;

pub struct Fanout<'a> {
    /// A btreemap with `channel_id` or `guild_id` as the key and a vec of
    /// tokio senders as the value.
    channels: BTreeMap<String, Vec<Sender<Event<'a>>>>,
}

impl Default for Fanout<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Fanout<'a> {
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
        }
    }

    pub fn add_sender<'b>(&'b mut self, channel: String, sender: Sender<Event<'a>>) {
        let senders = self.channels.entry(channel).or_insert(Vec::new());

        senders.push(sender);
    }

    pub fn remove_sender<'b>(&'b mut self, channel: String, sender: Sender<Event<'a>>) {
        let senders = self.channels.entry(channel).or_insert(Vec::new());

        senders.remove(
            senders
                .iter()
                .position(|s| s.same_channel(&sender))
                .unwrap(),
        );
    }
}
