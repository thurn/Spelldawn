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

//! Core game mutations. In general, functions in this module are the only ones
//! expected to append updates to [GameState::updates].

use data::card_state::{CardData, CardPosition, CardPositionKind};
use data::delegates::{
    self, CardMoved, DrawCardEvent, MoveCardEvent, PlayCardEvent, RaidEndEvent, Scope,
    StoredManaTakenEvent,
};
use data::game::GameState;
use data::primitives::{ActionCount, BoostData, CardId, ManaValue, Side};
use data::updates::GameUpdate;
use tracing::{info, instrument};

use crate::dispatch;

/// Overwrites the value of [CardData::boost_count] to match the provided
/// [BoostData]
#[instrument(skip(game))]
pub fn write_boost(game: &mut GameState, scope: Scope, data: BoostData) {
    info!(?scope, ?data, "write_boost");
    game.card_mut(data.card_id).data.boost_count = data.count;
    game.updates.push(GameUpdate::UpdateCard(data.card_id));
}

/// Set the boost count to zero for the card in `scope`
#[instrument(skip(game))]
pub fn clear_boost<T>(game: &mut GameState, scope: Scope, _: T) {
    info!(?scope, "clear_boost");
    game.card_mut(scope).data.boost_count = 0;
    game.updates.push(GameUpdate::UpdateCard(scope.card_id()));
}

/// Move a card to a new position. Detects cases like drawing cards, playing
/// cards, and shuffling cards back into the deck and fires events
/// appropriately.
#[instrument(skip(game))]
pub fn move_card(game: &mut GameState, card_id: CardId, new_position: CardPosition) {
    info!(?card_id, ?new_position, "move_card");
    let mut pushed_update = false;
    let old_position = game.card(card_id).position;
    game.move_card(card_id, new_position);

    dispatch::invoke_event(game, MoveCardEvent(CardMoved { old_position, new_position }));

    if old_position.in_deck() && new_position.in_hand() {
        dispatch::invoke_event(game, DrawCardEvent(card_id));
        game.updates.push(GameUpdate::DrawCard(card_id));
        pushed_update = true;
    }

    if !old_position.in_play() && new_position.in_play() {
        dispatch::invoke_event(game, PlayCardEvent(card_id));
    }

    if new_position.kind() == CardPositionKind::DeckUnknown {
        game.updates.push(GameUpdate::DestroyCard(card_id));
        pushed_update = true;
    }

    if !pushed_update {
        game.updates.push(GameUpdate::MoveCard(card_id));
    }
}

/// Give mana to the indicated player. Appends [GameUpdate::UpdateGameState].
#[instrument(skip(game))]
pub fn gain_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    info!(?side, ?amount, "gain_mana");
    game.player_mut(side).mana += amount;
    game.updates.push(GameUpdate::UpdateGameState);
}

/// Spends a player's mana. Appends [GameUpdate::UpdateGameState]. Panics if
/// sufficient action points are not available
#[instrument(skip(game))]
pub fn spend_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    info!(?side, ?amount, "spend_mana");
    assert!(game.player(side).mana >= amount, "Insufficient mana available");
    game.player_mut(side).mana -= amount;
    game.updates.push(GameUpdate::UpdateGameState);
}

/// Spends a player's action points. Appends [GameUpdate::UpdateGameState].
/// Panics if sufficient action points are not available.
#[instrument(skip(game))]
pub fn spend_action_points(game: &mut GameState, side: Side, amount: ActionCount) {
    info!(?side, ?amount, "spend_action_points");
    assert!(game.player(side).actions >= amount, "Insufficient action points available");
    game.player_mut(side).actions -= amount;
    game.updates.push(GameUpdate::UpdateGameState);
}

/// Takes *up to* `amount` stored mana from a card and gives it to the player
/// who owns this card. Panics if there is no stored mana available.233z
#[instrument(skip(game))]
pub fn take_stored_mana(game: &mut GameState, card_id: CardId, amount: ManaValue) {
    info!(?card_id, ?amount, "take_stored_mana");
    let available = game.card(card_id).data.stored_mana;
    assert!(available > 0, "No stored mana available!");
    let taken = std::cmp::min(available, amount);
    game.card_mut(card_id).data.stored_mana -= taken;
    dispatch::invoke_event(game, StoredManaTakenEvent(card_id));
    game.updates.push(GameUpdate::UpdateCard(card_id));
    gain_mana(game, card_id.side, taken);
}

/// Ends the current raid.
#[instrument(skip(game))]
pub fn set_raid_ended(game: &mut GameState) {
    info!("set_raid_ended");
    dispatch::invoke_event(game, RaidEndEvent(game.data.raid.expect("Active raid")));
    game.data.raid = None;
    game.updates.push(GameUpdate::EndRaid);
}
