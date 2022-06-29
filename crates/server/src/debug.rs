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

use std::collections::HashMap;

use adapters;
use anyhow::{bail, Result};
use data::agent_definition::AgentData;
use data::deck::Deck;
use data::fail;
use data::game::GameState;
use data::game_actions::DebugAction;
use data::primitives::{GameId, PlayerId, Side};
use data::with_error::WithError;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    ClientDebugCommand, CommandList, ConnectToGameCommand, CreateGameDebugOptions,
    CreateNewGameAction, GameAction, GameCommand, GameIdentifier, LoadSceneCommand, SceneLoadMode,
    SetPlayerIdentifierCommand,
};
use rules::{dispatch, mana, mutations};

use crate::database::Database;
use crate::requests;
use crate::requests::GameResponse;

pub fn handle_debug_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    action: DebugAction,
) -> Result<GameResponse> {
    match action {
        DebugAction::NewGame(side) => Ok(GameResponse {
            command_list: CommandList {
                commands: vec![GameCommand {
                    command: Some(Command::Debug(ClientDebugCommand {
                        debug_command: Some(DebugCommand::InvokeAction(GameAction {
                            action: Some(Action::CreateNewGame(CreateNewGameAction {
                                side: adapters::player_side(side),
                                opponent_id: Some(adapters::player_identifier(
                                    if player_id.value == 1 {
                                        PlayerId::new(2)
                                    } else {
                                        PlayerId::new(1)
                                    },
                                )),
                                debug_options: Some(CreateGameDebugOptions {
                                    deterministic: false,
                                    override_game_identifier: Some(GameIdentifier { value: 0 }),
                                    in_memory: false,
                                }),
                            })),
                        })),
                    })),
                }],
            },
            opponent_response: None,
        }),
        DebugAction::JoinGame => Ok(GameResponse {
            command_list: CommandList {
                commands: vec![GameCommand {
                    command: Some(Command::ConnectToGame(ConnectToGameCommand {
                        game_id: Some(GameIdentifier { value: 0 }),
                        scene_name: "Labyrinth".to_string(),
                    })),
                }],
            },
            opponent_response: None,
        }),
        DebugAction::ResetGame => {
            let game = load_game(database, game_id)?;
            reset_game(database, game_id)?;
            let commands = CommandList {
                commands: vec![GameCommand {
                    command: Some(Command::LoadScene(LoadSceneCommand {
                        scene_name: "Labyrinth".to_string(),
                        mode: SceneLoadMode::Single.into(),
                    })),
                }],
            };
            Ok(GameResponse {
                command_list: commands.clone(),
                opponent_response: Some((
                    if player_id == game.overlord.id { game.champion.id } else { game.overlord.id },
                    commands,
                )),
            })
        }
        DebugAction::AddMana(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                mana::gain(game, user_side, amount);
                Ok(())
            })
        }
        DebugAction::AddActionPoints(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).actions += amount;
                Ok(())
            })
        }
        DebugAction::AddScore(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).score += amount;
                Ok(())
            })
        }
        DebugAction::FlipViewpoint => Ok(GameResponse::from_commands(vec![
            Command::SetPlayerId(SetPlayerIdentifierCommand {
                id: Some(adapters::player_identifier(opponent_player_id(
                    database, player_id, game_id,
                )?)),
            }),
            Command::LoadScene(LoadSceneCommand {
                scene_name: "Labyrinth".to_string(),
                mode: SceneLoadMode::Single.into(),
            }),
        ])),
        DebugAction::SaveState(index) => {
            let mut game = load_game(database, game_id)?;
            game.id = GameId::new(u64::MAX - index);
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![]))
        }
        DebugAction::LoadState(index) => {
            let mut game = database.game(GameId::new(u64::MAX - index))?;
            game.id = game_id.with_error(|| "Expected GameId")?;
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
                scene_name: "Labyrinth".to_string(),
                mode: SceneLoadMode::Single.into(),
            })]))
        }
        DebugAction::SetAgent(side, state_predictor, agent) => {
            requests::handle_custom_action(database, player_id, game_id, |game, _user_side| {
                game.player_mut(side).agent = Some(AgentData { name: agent, state_predictor });
                Ok(())
            })
        }
    }
}

fn reset_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<()> {
    let current_game = load_game(database, game_id)?;
    let mut new_game = GameState::new(
        current_game.id,
        Deck {
            owner_id: current_game.overlord.id,
            identity: current_game.some_identity(Side::Overlord).expect("identity").name,
            cards: current_game
                .overlord_cards
                .iter()
                .filter(|c| {
                    c.id != current_game.some_identity(Side::Overlord).expect("identity").id
                })
                .fold(HashMap::new(), |mut acc, card| {
                    *acc.entry(card.name).or_insert(0) += 1;
                    acc
                }),
        },
        Deck {
            owner_id: current_game.champion.id,
            identity: current_game.some_identity(Side::Champion).expect("identity").name,
            cards: current_game
                .champion_cards
                .iter()
                .filter(|c| {
                    c.id != current_game.some_identity(Side::Champion).expect("identity").id
                })
                .fold(HashMap::new(), |mut acc, card| {
                    *acc.entry(card.name).or_insert(0) += 1;
                    acc
                }),
        },
        current_game.data.config,
    );
    dispatch::populate_delegate_cache(&mut new_game);
    mutations::deal_opening_hands(&mut new_game)?;
    database.write_game(&new_game)?;
    Ok(())
}

fn opponent_player_id(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
) -> Result<PlayerId> {
    let game = load_game(database, game_id)?;
    if player_id == game.overlord.id {
        Ok(game.champion.id)
    } else if player_id == game.champion.id {
        Ok(game.overlord.id)
    } else {
        fail!("ID must be present in game")
    }
}

fn load_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<GameState> {
    database.game(game_id.with_error(|| "GameId is required")?)
}
