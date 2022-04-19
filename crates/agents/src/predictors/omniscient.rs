// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The omniscient state predictor creates predictions with perfect information
//! about the opponent's cards.

use data::agent_definition::GameStatePredictorName;
use data::game::GameState;
use data::primitives::Side;
use linkme::distributed_slice;

use crate::core::types::{StatePrediction, StatePredictionIterator};
use crate::core::{GameStatePredictorPair, GAME_STATE_PREDICTORS};

pub fn initialize() {}

#[distributed_slice(GAME_STATE_PREDICTORS)]
static PREDICTOR: GameStatePredictorPair = (GameStatePredictorName::Omniscient, omniscient);

fn omniscient(game: &GameState, _side: Side) -> StatePredictionIterator {
    Box::new(vec![StatePrediction { score: 0.0, state: game.clone() }].into_iter())
}
