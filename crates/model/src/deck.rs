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

use crate::card_name::CardName;
use crate::primitives::Side;
use std::collections::HashMap;
use std::iter;

/// Represents a player deck outside of an active game
#[derive(Debug, Clone)]
pub struct Deck {
    /// Identity card for this deck
    pub identity: CardName,
    /// How many cards with each name are present in this deck?
    pub cards: HashMap<CardName, u32>,
}

impl Deck {
    /// Returns a vector which contains the identity card name, then repeats each [CardName] in
    /// this deck in alphabetical order a number of times equal to its deck count.
    pub fn card_names(&self) -> Vec<CardName> {
        let mut result = self
            .cards
            .iter()
            .flat_map(|(name, count)| iter::repeat(*name).take(*count as usize))
            .collect::<Vec<_>>();
        result.sort();
        result.insert(0, self.identity);
        result
    }
}
