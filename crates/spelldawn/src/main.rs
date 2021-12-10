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

#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::cloned_instead_of_copied)]
#![deny(clippy::copy_iterator)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::inconsistent_struct_constructor)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::integer_division)]
#![deny(clippy::let_underscore_drop)]
#![deny(clippy::let_underscore_must_use)]
#![deny(clippy::manual_ok_or)]
#![deny(clippy::map_flatten)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::multiple_inherent_impl)]
#![deny(clippy::needless_continue)]
#![deny(clippy::needless_for_each)]
#![deny(clippy::option_if_let_else)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::ref_option_ref)]
#![deny(clippy::string_to_string)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_self_imports)]
#![deny(clippy::unnested_or_patterns)]
#![deny(clippy::unused_self)]
#![deny(clippy::unwrap_in_result)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::use_self)]
#![deny(clippy::used_underscore_binding)]
#![deny(clippy::useless_let_if_seq)]
#![deny(clippy::wildcard_imports)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use model::card_definition::CardDefinition;
use model::events;
use model::events::{EventContext, GameEvent};
use model::game::GameState;
use model::primitives::{CardId, EventId, Side};
use tonic::{transport::Server, Request, Response, Status};

use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::{Spelldawn, SpelldawnServer};
use protos::spelldawn::{
    CommandList, GameCommand, GameId, GameRequest, GameView, RenderGameCommand,
};

use cards::ALL_CARDS;

#[derive(Default)]
pub struct GameService {}

#[tonic::async_trait]
impl Spelldawn for GameService {
    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = CommandList {
            commands: vec![GameCommand {
                command: Some(Command::RenderGame(RenderGameCommand {
                    game: Some(GameView {
                        game_id: Some(GameId { value: "GAME_ID".to_owned() }),
                        user: None,
                        opponent: None,
                        arena: None,
                        current_priority: 0,
                    }),
                })),
            }],
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "127.0.0.1:50052".parse().expect("valid address");
    let service = tonic_web::config()
        .allow_origins(vec!["127.0.0.1"])
        .enable(SpelldawnServer::new(GameService::default()));

    println!("Num CARDS {:?}", ALL_CARDS.len());

    let mut cards: Vec<CardDefinition> = vec![];
    for card_fn in ALL_CARDS {
        cards.push(card_fn());
    }

    let mut game = GameState::default();
    let context = EventContext { event_id: EventId(12), side: Side::Champion, this: CardId(4) };

    println!("Mana: {:?}", game.champion.state.mana);

    for card in cards {
        println!("{:?}", card);
        for handler in card.behavior.handlers {
            events::invoke_if_matching(&mut game, context, GameEvent::OnPlay, &handler.callback);
        }
    }

    println!("Mana: {:?}", game.champion.state.mana);
    println!("Server listening on {}", address);
    Server::builder().accept_http1(true).add_service(service).serve(address).await?;

    Ok(())
}
