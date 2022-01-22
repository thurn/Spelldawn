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

//! The debug panel provides tools for modifying the game state during
//! development. Typically these options should not be available to production
//! users.

use data::actions::{DebugAction, UserAction};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexStyle, FlexWrap, Node, PanelAddress};
use ui::components::{Button, Row};
use ui::core::{child, node};
use ui::panel::Panel;
use ui::{core, icons};

/// Renders the debug panel
pub fn render() -> Node {
    node(Panel {
        address: PanelAddress::DebugPanel,
        title: Some("Debug Controls".to_string()),
        width: 1024.0,
        height: 512.0,
        content: Row {
            name: "DebugButtons".to_string(),
            style: FlexStyle {
                wrap: FlexWrap::Wrap.into(),
                align_items: FlexAlign::Center.into(),
                justify_content: FlexJustify::Center.into(),
                ..FlexStyle::default()
            },
            children: vec![
                debug_button("Reset", UserAction::DebugAction(DebugAction::ResetGame)),
                debug_button("Fetch UI", UserAction::DebugAction(DebugAction::FetchStandardPanels)),
                debug_button(
                    format!("+10{}", icons::MANA),
                    UserAction::DebugAction(DebugAction::AddMana),
                ),
                debug_button(
                    format!("+{}", icons::ACTION),
                    UserAction::DebugAction(DebugAction::AddActionPoints),
                ),
                debug_button("+ Point", UserAction::DebugAction(DebugAction::AddScore)),
                debug_button("Turn", UserAction::DebugAction(DebugAction::SwitchTurn)),
                debug_button("Flip View", UserAction::DebugAction(DebugAction::FlipViewpoint)),
            ],
            ..Row::default()
        },
        show_close_button: true,
        ..Panel::default()
    })
}

fn debug_button(label: impl Into<String>, action: UserAction) -> Option<Node> {
    child(Button {
        label: label.into(),
        action: core::action(Some(action), None),
        style: FlexStyle { margin: core::all_px(8.0), ..FlexStyle::default() },
        ..Button::default()
    })
}
