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

using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class StackObjectDisplay : ObjectDisplay
  {
    const float LongPressTime = 1.0f;
    const float DragGestureScreenDistance = 10f;
    [SerializeField] float _singleElementY = 0.5f;
    Vector3? _mouseDownPosition;
    float? _mouseDownTime;

    protected override Vector3 CalculateObjectPosition(int index, int count) =>
      new(
        transform.position.x,
        transform.position.y + Mathf.Lerp(0f, 1f, YPosition(index, count)),
        transform.position.z);

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    public override bool CanHandleMouseDown() => true;

    public override void MouseDown()
    {
      _mouseDownPosition = Input.mousePosition;
      _mouseDownTime = Time.time;
    }

    public override void MouseDrag()
    {
      if (_mouseDownPosition is { } p && Vector3.Distance(Input.mousePosition, p) > DragGestureScreenDistance)
      {
        // Drag gesture
        _mouseDownTime = null;
      }

      if (_mouseDownTime is { } m && Time.time - m > LongPressTime)
      {
        LongPress();
        _mouseDownTime = null;
      }
    }

    protected virtual void LongPress()
    {
    }

    public override void MouseUp()
    {
      _mouseDownTime = null;
    }

    float YPosition(int index, int count) => count switch
    {
      0 => _singleElementY,
      1 => _singleElementY,
      2 => new[] { 0.4f, 0.6f }[index],
      3 => new[] { 0.3f, 0.5f, 0.7f }[index],
      _ => index / ((float)count - 1)
    };
  }
}