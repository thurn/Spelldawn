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

//! Core game mutations. In general, functions in this module append updates to
//! [GameState::updates]. Functions in this module panic if their preconditions
//! are not met, the higher-level game UI is responsible for ensuring this does
//! not happen.
//!
//! Generally, mutation functions are expected to invoke a
//! delegate event *after* performing their mutation to inform other systems
//! that game state has changed.

#[allow(unused)] // Used in rustdocs
use data::card_state::{CardData, CardPosition, CardPositionKind};
use data::delegates::{
    CardMoved, DrawCardEvent, MoveCardEvent, PlayCardEvent, RaidBeginEvent, RaidEndEvent,
    RevealCardEvent, Scope, StoredManaTakenEvent,
};
use data::game::{GameState, RaidData, RaidPhase};
use data::primitives::{ActionCount, BoostData, CardId, ManaValue, RaidId, RoomId, Side};
use data::prompt::{ActivateRoomAction, Prompt, PromptKind, PromptResponse};
use data::updates::GameUpdate;
use tracing::{info, instrument};

use crate::dispatch;

/// Move a card to a new position. Detects cases like drawing cards, playing
/// cards, and shuffling cards back into the deck and fires events appropriately
///
/// This function does *not* handle changing the 'revealed' status of the card,
/// the caller is responsible for updating that when the card moves to a public
/// game zone.
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

/// Updates the 'revealed' state of a card. Fires [RevealCardEvent] and appends
/// [GameUpdate::RevealCard] if the new state is revealed.
#[instrument(skip(game))]
pub fn set_revealed(game: &mut GameState, card_id: CardId, revealed: bool) {
    let current = game.card(card_id).data.revealed;

    game.card_mut(card_id).data.revealed = revealed;

    if !current && revealed {
        game.updates.push(GameUpdate::RevealCard(card_id));
        dispatch::invoke_event(game, RevealCardEvent(card_id));
    }
}

/// Give mana to the indicated player.
#[instrument(skip(game))]
pub fn gain_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    info!(?side, ?amount, "gain_mana");
    game.player_mut(side).mana += amount;
}

/// Spends a player's mana. Panics if sufficient mana is not available
/// [instrument(skip(game))]
pub fn spend_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    info!(?side, ?amount, "spend_mana");
    assert!(game.player(side).mana >= amount, "Insufficient mana available");
    game.player_mut(side).mana -= amount;
}

/// Spends a player's action points.
///
/// Panics if sufficient action points are not available.
#[instrument(skip(game))]
pub fn spend_action_points(game: &mut GameState, side: Side, amount: ActionCount) {
    info!(?side, ?amount, "spend_action_points");
    assert!(game.player(side).actions >= amount, "Insufficient action points available");
    game.player_mut(side).actions -= amount;
}

/// Takes *up to* `maximum` stored mana from a card and gives it to the player
/// who owns this card.
#[instrument(skip(game))]
pub fn take_stored_mana(game: &mut GameState, card_id: CardId, maximum: ManaValue) {
    info!(?card_id, ?maximum, "take_stored_mana");
    let available = game.card(card_id).data.stored_mana;
    let taken = std::cmp::min(available, maximum);
    game.card_mut(card_id).data.stored_mana -= taken;
    gain_mana(game, card_id.side, taken);
    dispatch::invoke_event(game, StoredManaTakenEvent(card_id));
}

/// Overwrites the value of [CardData::boost_count] to match the provided
/// [BoostData].
#[instrument(skip(game))]
pub fn write_boost(game: &mut GameState, scope: Scope, data: BoostData) {
    info!(?scope, ?data, "write_boost");
    game.card_mut(data.card_id).data.boost_count = data.count;
}

/// Set the boost count to zero for the card in `scope`.
#[instrument(skip(game))]
pub fn clear_boost<T>(game: &mut GameState, scope: Scope, _: T) {
    info!(?scope, "clear_boost");
    game.card_mut(scope.card_id()).data.boost_count = 0;
}

/// Sets the current prompt for the `side` player to the provided
/// [PromptResponse]. Appends [GameUpdate::UserPrompt]. Panics if a prompt is
/// already set for this player.
pub fn set_prompt(game: &mut GameState, side: Side, prompt: Prompt) {
    assert!(game.player(side).prompt.is_none(), "Player {:?} already has an active prompt", side);
    game.player_mut(side).prompt = Some(prompt);
    game.updates.push(GameUpdate::UserPrompt(side))
}

/// Clears shown prompts for both players. Appends [GameUpdate::ClearPrompts].
pub fn clear_prompts(game: &mut GameState) {
    game.overlord.prompt = None;
    game.champion.prompt = None;
    game.updates.push(GameUpdate::ClearPrompts);
}

/// Initiates a new raid on the given `room_id`. Panics if a raid is already
/// active. Appends [GameUpdate::InitiateRaid].
#[instrument(skip(game))]
pub fn initiate_raid(game: &mut GameState, room_id: RoomId) {
    info!(?room_id, "initiate_raid");
    assert!(game.data.raid.is_none(), "Raid is already active");
    let phase = if game.has_hidden_defenders(room_id) {
        set_prompt(
            game,
            Side::Overlord,
            Prompt {
                kind: PromptKind::ActivateRoomAction,
                responses: vec![
                    PromptResponse::ActivateRoomAction(ActivateRoomAction::Activate),
                    PromptResponse::ActivateRoomAction(ActivateRoomAction::Pass),
                ],
            },
        );
        RaidPhase::Activation
    } else {
        RaidPhase::Access
    };

    let raid =
        RaidData { target: room_id, raid_id: RaidId(game.data.next_raid_id), phase, active: false };
    game.data.next_raid_id += 1;
    game.data.raid = Some(raid);
    dispatch::invoke_event(game, RaidBeginEvent(raid));
    game.updates.push(GameUpdate::InitiateRaid(room_id));
}

/// Ends the current raid. Panics if no raid is currently active. Appends
/// [GameUpdate::EndRaid].
#[instrument(skip(game))]
pub fn end_raid(game: &mut GameState) {
    info!("end_raid");
    game.data.raid = None;
    dispatch::invoke_event(game, RaidEndEvent(game.data.raid.expect("Active raid")));
    game.updates.push(GameUpdate::EndRaid);
}
