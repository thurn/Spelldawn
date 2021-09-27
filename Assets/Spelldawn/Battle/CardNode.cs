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

using Spelldawn.Protos;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed record CardProps
  {
    public CardProps(CardId id)
    {
      Id = id;
    }

    public CardId Id { get; }
    public float Scale { get; init; } = 1.0f;
    public float? HandPosition { get; init; }
    public bool OverlayDim { get; init; }
  }

  public static class CardNode
  {
    public static Node? Render(CardView? cardView, CardProps props) => cardView?.CardCase switch
    {
      CardView.CardOneofCase.RevealedCard => RevealedCardNode.Render(cardView.RevealedCard, props),
      CardView.CardOneofCase.HiddenCard => HiddenCardNode.Render(cardView.HiddenCard),
      _ => null
    };
  }
}