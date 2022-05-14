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

//! Card definitions for the Project card type

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use data::text::{Keyword, Sentence};
use linkme::distributed_slice;
use rules::helpers::*;
use rules::mutations::OnZeroStored;
use rules::{abilities, mutations, text, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn gold_mine() -> CardDefinition {
    CardDefinition {
        name: CardName::GoldMine,
        cost: cost(4),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_43"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::unveil_at_dusk_then_store::<12>(),
            Ability {
                text: text![Keyword::Dusk, Keyword::Take(Sentence::Start, 3)],
                ability_type: AbilityType::Standard,
                delegates: vec![at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice);
                    alert(g, &s);
                })],
            },
        ],
        config: CardConfig::default(),
    }
}
