/*
    Copyright 2024 V.J. De Chico

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

use pulsar::{Producer, Pulsar, SerializeMessage, TokioExecutor, producer, proto};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use crate::{
    db::{actor::Actor, track::Track},
    state::State,
};

pub async fn get_producer(
    pulsar: &Pulsar<TokioExecutor>,
) -> Result<Producer<TokioExecutor>, crate::error::Error> {
    Ok(pulsar
        .producer()
        .with_topic("non-persistent://public/default/relay")
        .with_name("personality")
        .with_options(producer::ProducerOptions {
            schema: Some(proto::Schema {
                r#type: proto::schema::Type::String as i32,
                ..Default::default()
            }),
            ..Default::default()
        })
        .build()
        .await?)
}

#[derive(Serialize, Deserialize, Validate)]
pub enum BeamMessage {
    TrackCreate(Track),
    TrackDelete(Track),
    UserDelete(Actor),
}

impl SerializeMessage for BeamMessage {
    fn serialize_message(input: Self) -> Result<pulsar::producer::Message, pulsar::Error> {
        let payload =
            serde_json::to_vec(&input).map_err(|e| pulsar::Error::Custom(e.to_string()))?;
        Ok(pulsar::producer::Message {
            payload,
            ..Default::default()
        })
    }
}

pub async fn beam_out(event: BeamMessage, state: &State) -> Result<(), crate::error::Error> {
    let mut producer = get_producer(&state.plsr).await?;

    producer.send_non_blocking(event).await?.await?;

    Ok(())
}
