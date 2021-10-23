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

using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class Displayable : MonoBehaviour
  {
    [SerializeField] ObjectDisplay? _parent;
    [SerializeField] GameContext _gameContext;
    [SerializeField] SortingGroup? _sortingGroup;

    public ObjectDisplay? Parent
    {
      get => _parent;
      set => _parent = value;
    }

    public GameContext GameContext => Errors.CheckEnum(HasGameContext ? _gameContext : DefaultGameContext());

    public bool HasGameContext => _gameContext != GameContext.Unspecified;

    protected virtual GameContext DefaultGameContext() => GameContext.Unspecified;

    public void SetGameContext(GameContext gameContext, int? index = null)
    {
      Errors.CheckEnum(gameContext);

      if (_gameContext != gameContext)
      {
        if (_sortingGroup)
        {
          SortingOrder.Create(gameContext, index ?? 0).ApplyTo(_sortingGroup!);
        }

        var oldContext = _gameContext;
        _gameContext = gameContext;
        OnSetGameContext(oldContext, gameContext, index);
      }
    }

    protected abstract void OnSetGameContext(GameContext oldContext, GameContext newContext, int? index = null);
  }
}