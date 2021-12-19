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

use crate::card_helpers::*;
use crate::queries;
use model::card_definition::{Ability, AbilityText, AbilityType, Keyword};
use model::card_state::CardPosition;
use model::delegates::{Delegate, EventDelegate, QueryDelegate, Scope};
use model::game::GameState;
use model::primitives::{AttackValue, BoostData, CardId, ManaValue, Side};

/// Overwrites the value of [CardState::boost_count] to match the provided [BoostData]
fn write_boost(game: &mut GameState, scope: Scope, data: BoostData) {
    game.card_mut(data).data_mut().boost_count = data.count
}

/// Applies this card's `attack_boost` stat a number of times equal to its [CardState::boost_count]
fn add_boost(game: &GameState, scope: Scope, card_id: CardId, current: AttackValue) -> AttackValue {
    let boost_count = queries::boost_count(game, card_id);
    let bonus = queries::stats(game, card_id).attack_boost.expect("Expected boost").bonus;

    current + (boost_count * bonus)
}

/// Set the boost count to zero for the card in `scope`
fn clear_boost<T>(game: &mut GameState, scope: Scope, _: T) {
    game.card_mut(scope).data_mut().boost_count = 0
}

/// The standard weapon ability; applies an attack boost for the duration of a single encounter.
pub fn encounter_boost() -> Ability {
    Ability {
        text: AbilityText::TextFn(|g, s| {
            let boost = queries::stats(g, s).attack_boost.expect("attack_boost");
            vec![mana_cost_text(boost.cost), add_number(boost.bonus), text("Attack")]
        }),
        ability_type: AbilityType::Encounter,
        delegates: vec![
            Delegate::OnActivateBoost(EventDelegate::new(this_card, write_boost)),
            Delegate::GetAttackValue(QueryDelegate::new(this_card, add_boost)),
            Delegate::OnEncounterEnd(EventDelegate::new(always, clear_boost)),
        ],
    }
}

/// Store N mana in this card. Move it to the discard pile when the stored mana is depleted.
pub fn store_mana<const N: ManaValue>() -> Ability {
    Ability {
        text: AbilityText::Text(vec![keyword(Keyword::Play), keyword(Keyword::Store(N))]),
        ability_type: AbilityType::Standard,
        delegates: vec![
            Delegate::OnPlayCard(EventDelegate::new(this_card, |g, s, card_id| {
                g.card_mut(card_id).data_mut().stored_mana = N;
            })),
            Delegate::OnStoredManaTaken(EventDelegate::new(this_card, |g, s, card_id| {
                if g.card(card_id).data().stored_mana == 0 {
                    move_card(g, card_id, CardPosition::DiscardPile(s.side()))
                }
            })),
        ],
    }
}

/// Discard a random card from the hand of the `side` player, if there are any cards present.
pub fn discard_random_card(game: &mut GameState, side: Side) {
    if let Some(card_id) = game.random_card(CardPosition::Hand(side)) {
        move_card(game, card_id, CardPosition::DiscardPile(side));
    }
}

pub fn strike<const N: u32>() -> Ability {
    combat(
        AbilityText::Text(vec![keyword(Keyword::Combat), keyword(Keyword::Strike(N))]),
        |g, _, _| {
            for _ in 0..N {
                discard_random_card(g, Side::Champion);
            }
        },
    )
}

pub fn end_raid() -> Ability {
    combat(AbilityText::Text(vec![keyword(Keyword::Combat), text("End the raid.")]), |g, _, _| {
        set_raid_ended(g);
    })
}
