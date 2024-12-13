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

mod algo;
mod spider;

use serde::Deserialize;
use serde_valid::Validate;

/// The default number of posts fetched for recommendations.
fn default_limit() -> i32 {
    32
}

/// Get a certain number of Tracks based on several definite data points.
///
/// Using `thread_id` can be used to also fetch tracks from a parent post.
#[derive(Deserialize, Validate)]
pub struct Recommend {
    #[serde(default = "default_limit")]
    #[validate(maximum = 128)]
    #[validate(minimum = 12)]
    pub limit: i32,
    /// Thread ID.
    /// For data viability must be under 128 characters in length.
    /// For data variability must be at least 24 characters in length.
    #[serde(default)]
    #[validate(max_length = 128)]
    #[validate(min_length = 24)]
    pub thread_id: Option<String>,
    /// Actor ID.
    /// The actor who's frontpage should be returned.
    /// If `None` returns the newest crawled posts.
    pub actor_id: Option<String>,
}

#[tokio::main]
async fn main() {}
