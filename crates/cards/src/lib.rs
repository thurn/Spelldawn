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

//! Contains the definitions for all cards in the game.

use data::card_definition::CardDefinition;
use rules::DEFINITIONS;

pub mod artifacts;
pub mod champion_spells;
pub mod decklists;
pub mod initialize;
pub mod minions;
pub mod overlord_spells;
pub mod projects;
pub mod schemes;
pub mod test_cards;
pub mod weapons;

pub fn insert_definition(function: fn() -> CardDefinition) {
    DEFINITIONS.insert(function);
}
