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

use rust_bert::pipelines::keywords_extraction::KeywordExtractionModel;

/// Returns a list of up to 35 persistent keywords present inside of the content.
pub fn topics_from_content(content: &str) -> Vec<String> {
    // TODO: handle errors instead of panicking

    // TODO: keep this in state?
    let model = KeywordExtractionModel::new(Default::default()).unwrap();

    let predictions = model.predict(&[content]).unwrap();
    let mut pred = predictions[0].clone();

    pred.sort_by(|a, b| a.score.total_cmp(&b.score));
    pred.into_iter().take(35).map(|itm| itm.text).collect()
}
