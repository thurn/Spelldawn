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


#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Net.Http;
using System.Text;
using Grpc.Core;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using Grpc.Net.Compression;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using CompressionLevel = System.IO.Compression.CompressionLevel;
using Random = UnityEngine.Random;

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    static readonly string ServerAddress = "http://localhost:50052";

    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];

    readonly Protos.Spelldawn.SpelldawnClient _client = new(GrpcChannel.ForAddress(
      ServerAddress, new GrpcChannelOptions
      {
        HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
        Credentials = ChannelCredentials.Insecure,
        CompressionProviders = new List<ICompressionProvider>
        {
          new GzipCompressionProvider(CompressionLevel.Optimal)
        }
      }));

    readonly Queue<GameAction> _actionQueue = new();

    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _currentPriority;
    [SerializeField] bool _currentlyHandlingAction;
    Clickable? _lastClicked;

    public bool OfflineMode { get; private set; }

    public void Initialize()
    {
      Plugin.Initialize(PlayerPrefs.GetInt(Preferences.InMemory) > 0);
    }

    public PlayerName CurrentPriority
    {
      set => _currentPriority = value;
    }

    public void Connect(GameIdentifier? gameIdentifier, bool offlineMode)
    {
      OfflineMode = offlineMode;
      ConnectToRulesEngine(gameIdentifier);
    }

    public void HandleAction(GameAction action)
    {
      if (!CanExecuteAction(action.ActionCase))
      {
        var message = new StringBuilder();
        message.Append($"Error: User cannot currently perform action {action}");
        throw new InvalidOperationException(message.ToString());
      }

      _actionQueue.Enqueue(action);
    }

    /// <summary>
    /// Can the user currently zoom a card that exists in the provided GameContext.
    /// </summary>
    public bool CanInfoZoom(Displayable displayable, GameContext gameContext)
    {
      if (_registry.DocumentService.IsAnyPanelOpen())
      {
        return false;
      }

      switch (gameContext)
      {
        case GameContext.ArenaRaidParticipant:
        case GameContext.RaidParticipant:
          // If a card is a top-level raid participant, it can be info zoomed. However if a card is *part* of
          // a parent display that is participating in a raid (e.g. it is part of the discard pile that is 
          // being targeted), then it cannot be info zoomed and the long-press browser is used instead.
          return displayable.Parent == _registry.RaidService.RaidParticipants;
        case GameContext.Browser:
        case GameContext.RewardBrowser:
        case GameContext.LongPressBrowser:  
          return true;
        case GameContext.Deck:
        case GameContext.DiscardPile:
          return false;
        default:
          return !AnyOverlayOpen();
      }
    }

    /// <summary>
    /// Can the user *start* performing an action such as dragging a card out of their hand or dragging a raid arrow.
    /// This is allowed more leniently than actually *performing* an action as defined by
    /// <see cref="CanExecuteAction"/> below.
    /// </summary>
    public bool CanInitiateAction() => !_registry.CardService.CurrentlyDragging &&
                                       !AnyOverlayOpen() &&
                                       !_registry.DocumentService.IsAnyPanelOpen();

    public bool AnyOverlayOpen() => _registry.RaidOverlay.Enabled ||
                                    _registry.InterfaceOverlay.Enabled ||
                                    _registry.LongPressOverlay.Enabled;
    
    /// <summary>
    /// Can the user currently perform a game action of the provided type?
    /// </summary>
    public bool CanExecuteAction(GameAction.ActionOneofCase actionType) => actionType switch
    {
      GameAction.ActionOneofCase.StandardAction => CanAct(
        allowInOverlay: true,
        actionPointRequired: false,
        allowWithPanelOpen: true),
      GameAction.ActionOneofCase.FetchPanel => true,
      GameAction.ActionOneofCase.CreateNewGame => true,
      GameAction.ActionOneofCase.SyncAction => true,
      GameAction.ActionOneofCase.GainMana => CanAct(),
      GameAction.ActionOneofCase.DrawCard => CanAct(),
      GameAction.ActionOneofCase.PlayCard => CanAct(),
      GameAction.ActionOneofCase.LevelUpRoom => CanAct(),
      GameAction.ActionOneofCase.InitiateRaid => CanAct(),
      _ => false
    };

    bool CanAct(bool allowInOverlay = false, bool actionPointRequired = true, bool allowWithPanelOpen = false) =>
      !_registry.CardService.CurrentlyDragging &&
      (allowWithPanelOpen || !_registry.DocumentService.IsAnyPanelOpen()) &&
      (allowInOverlay || !AnyOverlayOpen()) &&
      (allowInOverlay || !_registry.RaidService.RaidActive) &&
      (!actionPointRequired || _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions > 0);

    void Update()
    {
      if (_actionQueue.Count > 0 && !_currentlyHandlingAction)
      {
        _currentlyHandlingAction = true;
        StartCoroutine(HandleActionAsync(_actionQueue.Dequeue()));
      }

      var userLight = _registry.ActiveLightForPlayer(PlayerName.User);
      var opponentLight = _registry.ActiveLightForPlayer(PlayerName.Opponent);

      switch (_currentPriority)
      {
        case PlayerName.User when CanExecuteAction(GameAction.ActionOneofCase.PlayCard):
          userLight.SetActive(true);
          break;
        case PlayerName.Opponent:
          opponentLight.SetActive(true);
          break;
        case PlayerName.Unspecified:
        default:
          userLight.SetActive(false);
          opponentLight.SetActive(false);
          break;
      }

      switch (Input.GetMouseButton(0))
      {
        case true when _lastClicked:
          _lastClicked!.MouseDrag();
          break;
        case true when !_lastClicked:
          _lastClicked = FireMouseDown();
          break;
        case false when _lastClicked:
          var last = _lastClicked;
          _lastClicked = null; // Do this first in case MouseUp() throws
          last!.MouseUp();
          break;
      }

      var pollCommands = Plugin.Poll();
      if (pollCommands != null)
      {
        StartCoroutine(_registry.CommandService.HandleCommands(pollCommands));
      }
    }

    Clickable? FireMouseDown()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      Clickable? fired = null;

      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var clickable = hit.collider.GetComponent<Clickable>();
        if (clickable)
        {
          if (!fired)
          {
            var consumed = clickable.MouseDown();
            if (consumed)
            {
              fired = clickable;
            }
          }
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return fired;
    }

    async void ConnectToRulesEngine(GameIdentifier? gameId)
    {
      if (gameId == null)
      {
        Debug.Log("No active game");
        return;
      }

      var request = new ConnectRequest
      {
        GameId = gameId,
        PlayerId = _registry.GameService.PlayerId,
      };

      if (OfflineMode)
      {
        Debug.Log($"Connecting to Offline Game {request.GameId.Value}");
        StartCoroutine(ConnectToOfflineGame(request));
      }
      else
      {
        // TODO: Android in particular seems to hang for multiple minutes when the server can't be reached?
        Debug.Log($"Connecting to {ServerAddress} with game {request.GameId.Value}");
        using var call = _client.Connect(request);

        while (await call.ResponseStream.MoveNext())
        {
          if (this != null)
          {
            var commands = call.ResponseStream.Current;
            StartCoroutine(_registry.CommandService.HandleCommands(commands));
          }
        }
      }
    }

    /// <summary>Connects to an existing offline game, handling responses.</summary>
    public IEnumerator ConnectToOfflineGame(ConnectRequest request)
    {
      var commands = Plugin.Connect(request);
      if (commands != null)
      {
        yield return _registry.CommandService.HandleCommands(commands);
      }
    }

    IEnumerator HandleActionAsync(GameAction action)
    {
      StartCoroutine(ApplyOptimisticResponse(action));
      if (action.ActionCase == GameAction.ActionOneofCase.StandardAction && action.StandardAction.Payload.Length == 0)
      {
        // No need to send empty payload to server
        _currentlyHandlingAction = false;
        yield break;
      }

      // Introduce simulated server delay
      yield return new WaitForSeconds(Random.Range(0f, 0.5f));

      // Send to server
      var request = new GameRequest
      {
        Action = action,
        GameId = _registry.GameService.CurrentGameId,
        PlayerId = _registry.GameService.PlayerId,
      };

      if (OfflineMode)
      {
        yield return _registry.CommandService.HandleCommands(Plugin.PerformAction(request));
      }
      else
      {
        var call = _client.PerformActionAsync(request);
        var task = call.GetAwaiter();
        yield return new WaitUntil(() => task.IsCompleted);

        switch (call.GetStatus().StatusCode)
        {
          case StatusCode.OK:
            yield return _registry.CommandService.HandleCommands(task.GetResult());
            break;
          case StatusCode.Unavailable:
            Debug.LogError($"Server {ServerAddress} is not available! Attempting to fall back to offline.");
            yield return _registry.CommandService.HandleCommands(Plugin.PerformAction(request));
            break;
          default:
            Debug.LogError($"Error connecting to {ServerAddress}: {call.GetStatus().Detail}");
            break;
        }
      }

      _currentlyHandlingAction = false;
    }

    IEnumerator ApplyOptimisticResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.StandardAction:
          _registry.StaticAssets.PlayButtonSound();
          if (action.StandardAction.Update is { } update)
          {
            yield return _registry.CommandService.HandleCommands(update);
          }

          break;
        case GameAction.ActionOneofCase.DrawCard:
          _registry.StaticAssets.PlayDrawCardStartSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.CardService.DrawOptimisticCard();
          break;
        case GameAction.ActionOneofCase.PlayCard:
          yield return HandlePlayCard(action.PlayCard);
          break;
        case GameAction.ActionOneofCase.GainMana:
          _registry.StaticAssets.PlayAddManaSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.ManaDisplayForPlayer(PlayerName.User).GainMana(1);
          break;
        case GameAction.ActionOneofCase.InitiateRaid:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.InitiateRaid.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.InitiateRaid
            }
          });
          break;
        case GameAction.ActionOneofCase.LevelUpRoom:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.LevelUpRoom.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.LevelUpRoom
            }
          });
          break;
        default:
          yield break;
      }
    }

    IEnumerator HandlePlayCard(PlayCardAction action)
    {
      var card = _registry.CardService.FindCard(action.CardId);
      _registry.StaticAssets.PlayWhooshSound();
      if (card.ReleasePosition == null)
      {
        yield break;
      }

      var position = card.ReleasePosition;

      if (position.PositionCase == ObjectPosition.PositionOneofCase.Room)
      {
        var room = action.Target.RoomId;
        Errors.CheckArgument(room != RoomIdentifier.Unspecified, "No RoomId target provided!");
        // Move to targeted room
        var newPosition = new ObjectPosition();
        newPosition.MergeFrom(position);
        newPosition.Room.RoomId = room;
        position = newPosition;
      }

      yield return _registry.ObjectPositionService.MoveGameObject(card, position);
    }
  }

  /// <summary>
  /// You can use this type instead of 'GzipCompressionProvider' above to log the size of server payloads before
  /// decompression.
  /// </summary>
  // ReSharper disable once UnusedType.Global
  sealed class DebugGzipCompressionProvider : ICompressionProvider
  {
    readonly GzipCompressionProvider _wrappedProvider;

    public DebugGzipCompressionProvider(CompressionLevel defaultCompressionLevel)
    {
      _wrappedProvider = new GzipCompressionProvider(defaultCompressionLevel);
    }

    public Stream CreateCompressionStream(Stream stream, CompressionLevel? compressionLevel) =>
      _wrappedProvider.CreateCompressionStream(stream, compressionLevel);

    public Stream CreateDecompressionStream(Stream stream)
    {
      Debug.Log($">>> Decompressing: {stream.Length}");
      return _wrappedProvider.CreateDecompressionStream(stream);
    }

    public string EncodingName => _wrappedProvider.EncodingName;
  }
}