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

//! Card definitions for the Weapon card type

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, TargetRequirement};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use data::text::{Keyword, Sentence};
use data::utils;
use linkme::distributed_slice;
use rules::card_text::text;
use rules::helpers::*;
use rules::mutations::OnEmpty;
use rules::{abilities, mutations, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn lodestone() -> CardDefinition {
    CardDefinition {
        name: CardName::Lodestone,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_78"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            abilities::activated_take_mana::<2>(actions(1)),
        ],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn sanctum_passage() -> CardDefinition {
    CardDefinition {
        name: CardName::SanctumPassage,
        cost: cost(2),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_77"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!(
                "The first time each turn you access the Sanctum, access 1 additional card."
            ),
            ability_type: silent(),
            delegates: vec![
                on_raid_access_start(face_up_in_play, |g, s, raid_id| {
                    once_per_turn(g, s, raid_id, save_raid_id);
                }),
                add_sanctum_access::<1>(matching_raid),
            ],
        }],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn accumulator() -> CardDefinition {
    CardDefinition {
        name: CardName::Accumulator,
        cost: cost(3),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_76"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text!(Keyword::SuccessfulRaid, Keyword::Store(1)),
                ability_type: alert(),
                delegates: vec![on_raid_success(face_up_in_play, |g, s, _| {
                    add_stored_mana(g, s.card_id(), 1);
                })],
            },
            Ability {
                text: text!(Keyword::Store(1), ", then take all stored mana."),
                ability_type: AbilityType::Activated(actions(1), TargetRequirement::None),
                delegates: vec![on_activated(|g, s, activated| {
                    let mana = add_stored_mana(g, s.card_id(), 1);
                    mutations::take_stored_mana(g, activated.card_id(), mana, OnEmpty::Ignore);
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
fn mystic_portal() -> CardDefinition {
    CardDefinition {
        name: CardName::MysticPortal,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_75"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            Ability {
                text: text!(
                    "Raid an",
                    Keyword::InnerRoom(Sentence::Internal),
                    "you have not raided this turn.",
                    "If successful,",
                    Keyword::Take(Sentence::Internal, 3)
                ),
                ability_type: AbilityType::Activated(
                    actions(1),
                    TargetRequirement::TargetRoom(|g, ability_id, room_id| {
                        is_inner_room(room_id)
                            && utils::is_false(|| {
                                Some(
                                    *g.ability_state(ability_id)?.room_turns.get(&room_id)?
                                        == g.data.turn,
                                )
                            })
                    }),
                ),
                delegates: vec![
                    on_activated(|g, s, activated| {
                        initiate_raid(g, s, activated.target);
                    }),
                    on_raid_start(face_up_in_play, |g, s, raid_start| {
                        let turn = g.data.turn;
                        g.ability_state_mut(s.ability_id())
                            .room_turns
                            .insert(raid_start.target, turn);
                    }),
                    on_raid_success(matching_raid, |g, s, _| {
                        mutations::take_stored_mana(g, s.card_id(), 3, OnEmpty::MoveToDiscard)
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}
