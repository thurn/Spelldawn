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

using Spelldawn.Services;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class LinearObjectDisplay : ObjectDisplay
  {
    [SerializeField] float _width;
    [SerializeField] float _initialSpacing;
    [SerializeField] float _cardSize;
    [SerializeField] float _rotation = 270;
    [SerializeField] Registry _registry = null!;

    protected override Registry Registry => _registry;

    protected override GameContext DefaultGameContext() => Game.GameContext.Arena;

    protected override Vector3 CalculateObjectPosition(int index, int count) =>
      transform.position + new Vector3(CalculateXOffset(_width, _initialSpacing, _cardSize, index, count), 0, 0);

    protected override Vector3? CalculateObjectRotation(int index, int count) =>
      new Vector3(x: _rotation, y: 0, 0);

    public static float CalculateXOffset(
      float width,
      float initialSpacing,
      float cardWidth,
      int index,
      int count,
      float minOffsetMultiplier = 1f,
      float maxOffsetMultiplier = 1f)
    {
      var availableWidth = Mathf.Min(width, (cardWidth + initialSpacing) * count);
      var offset = (availableWidth / 2f - cardWidth / 2f);

      return count switch
      {
        0 or 1 => 0,
        _ => Mathf.Lerp(-offset * minOffsetMultiplier, offset * maxOffsetMultiplier, index / (count - 1f))
      };
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, 0), radius: 1);
      Gizmos.DrawSphere(transform.position, radius: 1);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, 0), radius: 1);
    }
  }
}