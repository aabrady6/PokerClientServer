<template>
  <div class="app_container" :class="{ unselectable: showEndMenu }">
    <Login
      v-if="!playerNameEntered"
      :playerName="playerName"
      :playerNameEntered="playerNameEntered"
      @loggedIn="handleLogin"
    />
    <div v-else-if="!isInLobby">
      {{ notInLobbyText }}
    </div>
    <template v-else>
      <div class="action_info">
        {{ currentAction }}
      </div>
      <Table
        :players="players"
        :communityCards="communityCards"
        :pot="pot"
        :playerName="playerName"
        @click="onCardClick"
      />
      <Controls
        v-if="isCurrentPlayer && !discardRound"
        class="controls"
        :controls="controls"
        @click="onControlsClick"
      />
      <div
        v-if="isCurrentPlayer && selectCardsActive && discardRound"
        class="control_button"
        @click="submitOnClick"
      >
        SUBMIT
      </div>
      <div v-if="winners.length" class="control_button" @click="clickEndMenu">
        END MENU
      </div>
      <div
        v-if="isDealer"
        class="control_button"
        :class="{ unavailable: !canClickStartGame }"
        @click="startGameClick"
      >
        START GAME
      </div>
      <div class="stats_menu">
        <span
          v-if="seeStats !== true"
          class="control_button"
          @click="openStatsMenu"
        >
          STATS
        </span>
        <div v-else class="stats_menu_open">
          <div class="close_btn" @click="closeStatsMenu">X</div>
          <Stats
            :playerData="playerData"
            :gamesData="gamesData"
            :singleGameData="singleGameData"
            @click="clickStatsMenu"
            @back="onStatsBack"
          />
        </div>
      </div>
      <div class="help_menu">
        <span
          v-if="seeHelp !== true"
          class="control_button"
          @click="openHelpMenu"
        >
          HELP
        </span>
        <div v-else class="help_menu_open">
          <div class="close_btn" @click="closeHelpMenu">X</div>
          <Help />
        </div>
      </div>
      <div v-if="showEndMenu && !isDealerChoiceSpectator" class="end_menu">
        <div class="close_btn" @click="closeEndMenu">X</div>
        <EndRoundScreen v-bind="endMenuData" @click="submitEndRound" />
      </div>
      <div v-if="showJoinGameMenu" class="join_game_menu">
        <EndRoundScreen v-bind="joinGameMenuData" @click="submitJoinGame" />
      </div>
    </template>
  </div>
</template>

<script setup>
import Login from "./components/Login.vue";
import Table from "./components/Table.vue";
import Controls from "./components/Controls.vue";
import EndRoundScreen from "./components/EndRoundScreen.vue";
import Stats from "./components/Stats.vue";
import Help from "./components/Help.vue";
import { computed, ref, watch } from "vue";

const RUST_SERVER_IP = import.meta.env.VITE_RUST_SERVER_IP || "localhost";
const RUST_SERVER_PORT = import.meta.env.VITE_RUST_SERVER_PORT || "8080";
const NODE_SERVER_IP = import.meta.env.VITE_NODE_SERVER_ID || "localhost";
const NODE_SERVER_PORT = import.meta.env.VITE_NODE_SERVER_PORT || "3000";

console.log("RUST_SERVER_IP:", import.meta.env.VITE_RUST_SERVER_IP);
console.log("RUST_SERVER_PORT:", import.meta.env.VITE_RUST_SERVER_PORT);


const players = ref([]);
const communityCards = ref([]);
const pot = ref(500);
const highestBet = ref(0);
const playerName = ref("");
const currentPlayer = ref(null);
const playerNameEntered = ref(false);
const minRaise = ref(0);
const maxRaise = ref(0);
const discardRound = ref(false);
const demoMode = ref("");
const winners = ref([]);
const gameStarted = ref(false);
const canClickStartGame = ref(false);
const maxPlayers = ref(0);
const showEndMenu = ref(false);
const showJoinGameMenu = ref(false);
const endMenuData = ref(null);
const joinGameMenuData = ref(null);
const seeStats = ref(false);
const playerData = ref([]);
const gamesData = ref([]);
const singleGameData = ref([]);
const seeHelp = ref(false);
const canMakeLobbyAction = ref(false);
const dealerIdx = ref(0);
const gameVariant = ref("");
const spectators = ref([]);
const dealerChoiceSpectators = ref([]);
const lobby = ref([]);
const selectCardsActive = ref(true);
const currentAction = ref("Waiting to Start a Game...");
const notInLobbyText = ref("Waiting for Game to End...");

const isSpectator = computed(() => (
  spectators.value.length
    ? !!spectators.value.find(p => p.player_name == playerName.value)
    : false
));
const isDealerChoiceSpectator = computed(() => (
  dealerChoiceSpectators.value.length
    ? !!dealerChoiceSpectators.value.find(p => p.player_name == playerName.value)
    : false
));
const isInLobby = computed(() => (
  lobby.value.length
    ? !!lobby.value.find(p => p.player_name == playerName.value)
    : false
));

const isDealer = computed(() => (
  players.value[dealerIdx.value]?.name === playerName.value
));

const winnersText = computed(() => {
  if (winners.value.length) {
    let text = "";
    text += winners.value.join(", ");
    text += " won the game!";
    return text;
  } else {
    return "";
  }
});

const isCurrentPlayer = computed(() => {
  return (
    currentPlayer.value && currentPlayer.value.player_name === playerName.value
  );
});

watch(() => isCurrentPlayer.value, (newValue) => {
  if (newValue) {
    console.log("✅ You ARE the current player.");
  } else {
    console.log("❌ You are NOT the current player.");
  }
});

const controls = computed(() => {
  const currentPlayer = players.value.find(
    (player) => player.name === playerName.value
  );
  if (!currentPlayer) {
    return [];
  }

  const isCurrentPlayerTurn = isCurrentPlayer.value;

  if (!isCurrentPlayerTurn) {
    return [];
  }

  if (showEndMenu.value) {
    return [];
  }

  return [
    {
      name: "Demo Mode",
      visible: demoMode.value === "inactive",
    },
    {
      name: "Fold",
      visible: !discardRound.value,
    },
    {
      name: "Check",
      visible:
        !discardRound.value &&
        (highestBet.value === 0 || currentPlayer.betCash == highestBet.value),
    },
    {
      name: "Call",
      visible:
        !discardRound.value &&
        highestBet.value > 0 &&
        currentPlayer.totalCash >= highestBet.value - currentPlayer.betCash &&
        currentPlayer.betCash !== highestBet.value,
    },
    {
      name: "Raise",
      visible:
        !discardRound.value &&
        currentPlayer.totalCash > 0 &&
        currentPlayer.totalCash + currentPlayer.betCash >= minRaise.value,
      needsAmount: true,
      amountMin: minRaise.value,
      amountMax: maxRaise.value,
    },
  ];
});

const formatCard = (card) => {
  const valueMap = {
    Ace: "A",
    Two: "2",
    Three: "3",
    Four: "4",
    Five: "5",
    Six: "6",
    Seven: "7",
    Eight: "8",
    Nine: "9",
    Ten: "T",
    Jack: "J",
    Queen: "Q",
    King: "K",
  };

  const suitMap = {
    Heart: "H",
    Diamond: "D",
    Club: "C",
    Spade: "S",
  };

  return `${valueMap[card.value]}${suitMap[card.suit]}`;
};

const onCardClick = (card) => {
  if (!selectCardsActive.value) {
    return;
  }

  const formattedCard = typeof card === "object" ? formatCard(card) : card;

  const player = players.value.find(
    (player) => player.name === playerName.value
  );
  if (!player) {
    console.error("Player not found:", playerName.value);
    return;
  }

  const clickedCard = player.faceDownCards.find(
    (fdcard) => fdcard[0] === formattedCard
  );
  if (!clickedCard) {
    console.error("Card not found in player's hand:", formattedCard);
    return;
  }

  clickedCard[1] = !clickedCard[1];
};

const submitOnClick = () => {
  if (!discardRound.value) {
    return;
  }

  selectCardsActive.value = false;

  const player = players.value.find(
    (player) => player.name === playerName.value
  );
  if (!player) {
    console.error("Player not found:", playerName.value);
    return;
  }

  const selectedCardIndices = player.faceDownCards
    .map((fdcard, index) => (fdcard[1] ? index : -1))
    .filter((index) => index !== -1);

  const payload = {
    type: "DiscardAction",
    player_name: playerName.value,
    card_index: selectedCardIndices.length === 0 ? [] : selectedCardIndices,
  };

  socket.send(JSON.stringify(payload));
  console.log("Sent action:", payload);
};

const onControlsClick = (message) => {
  const { action, betAmount } = message;

  if (action === "Demo Mode") {
    const demoModeMessage = {
      type: "DemoMode",
      player_name: playerName.value,
      status: "active",
    };

    socket.send(JSON.stringify(demoModeMessage));
    console.log("Sent demo mode message:", demoModeMessage);
    return;
  }

  const betAmountNumber = Number.isNaN(Number(betAmount))
    ? 0
    : Math.max(0, betAmount);

  const payload = {
    type: "PlayerAction",
    action,
    player_name: playerName.value,
    bet_amount: betAmountNumber,
  };

  socket.send(JSON.stringify(payload));
  console.log("Sent action:", payload);
};

async function startGameClick() {
  if (canClickStartGame.value) {
    if (players.value.length >= 2 && players.value.length <= maxPlayers.value) {
      try {
          const response = await fetch(`http://${NODE_SERVER_IP}:${NODE_SERVER_PORT}/startgame`, {
              method: "GET",
          });

          if (!response.ok) {
              const error = await response.text();
              throw new Error(error);
          }

          const data = await response.json();
          gameStarted.value = true;
          canClickStartGame.value = false;
      } catch (error) {
          console.error("Start game error:", error);
          alert(`Start game failed: ${error.message}`);
      }
    }
  }
}

const clickEndMenu = () => {
  const headerTextStart = (
    winnersText.value == ""
      ? `Next table is a ${gameVariant.value} game!`
      : winnersText.value
  );
  const winnersTextLocal = (
    canMakeLobbyAction.value
      ? headerTextStart
      : headerTextStart + "\nWaiting for dealer action..."
  );
  const tableOptionsLocal = (
    canMakeLobbyAction.value
      ? ["Join table", "Spectate game", "Leave table"]
      : []
  );
  const data = {
      headerText: winnersTextLocal,
      tableOptions: tableOptionsLocal,
      dealerOptions: ["Five-Card Draw", "Seven-Card Stud", "Texas Hold 'Em"],
      isDealer: isDealer.value,
    };
    showEndMenu.value = true;
    endMenuData.value = data;
}

async function clickStatsMenu(message) {
  if (seeStats.value) {
    const payload = {
      type: "StatsMenu",
      ...message,
    };

    try {
      const response = await fetch(
        `http://${NODE_SERVER_IP}:${NODE_SERVER_PORT}/stats?type=${payload.type}&stats_menu_type=${payload.stats_menu_type}&selected_option=${payload.selected_option}`,
        {
          method: "GET",
        }
      );

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error);
      }

      const data = await response.json();

      if (data.player_data) {
        const inner_data = JSON.parse(data.player_data);
        playerData.value = inner_data;
      }

      if (data.games_data) {
        const inner_data = JSON.parse(data.games_data);
        gamesData.value = inner_data;
      }

      if (data.single_game_data) {
        const inner_data = JSON.parse(data.single_game_data);
        singleGameData.value = inner_data;
      }
    } catch (error) {
      console.error("Get stats error:", error);
      alert(`Get stats failed: ${error.message}`);
    }
  }
}

const onStatsBack = (statsToRemove) => {
  if (statsToRemove == "single_game") {
    singleGameData.value = [];
  } else if (statsToRemove == "games") {
    gamesData.value = [];
  } else if (statsToRemove == "player") {
    playerData.value = [];
  }
};

const openStatsMenu = () => {
  seeStats.value = true;
  playerData.value = [];
  gamesData.value = [];
  singleGameData.value = [];
};

const closeStatsMenu = () => {
  seeStats.value = false;
};

const closeEndMenu = () => {
  showEndMenu.value = false;
};

const openHelpMenu = () => {
  seeHelp.value = true;
};

const closeHelpMenu = () => {
  seeHelp.value = false;
};

const submitEndRound = (message) => {
  if (message.option === "Leave table") {
    notInLobbyText.value = "Thanks for Playing! BYE BYE!!!";
  }

  const payload = {
        type: "EndRound",
        player_name: playerName.value,
        ...message,
    };
  socket.send(JSON.stringify(payload));
  console.log("Send end option:", payload);
  showEndMenu.value = false;
  winners.value = [];
}

const submitJoinGame = (message) => {
  if (message.option === "Leave table") {
    notInLobbyText.value = "Thanks for Playing! BYE BYE!!!";
  }

  const payload = {
        type: "EndRound",
        player_name: playerName.value,
        ...message,
    };
  socket.send(JSON.stringify(payload));
  console.log("Send join game option:", payload);
  showJoinGameMenu.value = false;
}

// Set up WebSocket connection
const socket = new WebSocket(`ws://${RUST_SERVER_IP}:${RUST_SERVER_PORT}/ws/`);
socket.onerror = (error) => console.error("WebSocket error:", error);
socket.onclose = () => console.log("WebSocket connection closed");
// Add to WebSocket open handler
socket.onopen = () => {
  console.log("WebSocket connection opened");
  // Request full state immediately after connection
  // fetchGameState();
};

// Modify message handler for better debugging
socket.onmessage = (event) => {
    try {
        const data = JSON.parse(event.data);

        // Update community cards
        if (data.community_cards?.cards) {
          communityCards.value = data.community_cards.cards.map((card) => [
            formatCard(card),
            false,
          ]);
        }

        if (data.lobby) {
          lobby.value = data.lobby;
          if (data.lobby.length == 0) {
            resetClientState();
            return;
          }
        }

        if (data.dealer || (data.dealer == 0)) {
          console.log("Updating dealer index to: ", data.dealer);
          dealerIdx.value = data.dealer;
        }

        // Update players
        if (data.players) {
            if (data.players.length >= 2 && data.players.length <= data.max_players && !gameStarted.value && !data.winner.length) {
              console.log("~~~~~~~~~~~~~~~~~~~~~~~~");
              console.log("CAN CLICK START GAME!!!");
              console.log("players length: ", data.players.length);
              console.log("max players: ", data.max_players);
              console.log("winners length: ", data.winner.length);
              console.log("~~~~~~~~~~~~~~~~~~~~~~~~");
              canClickStartGame.value = true;
            } else {
              console.log("CANNOT CLICK START GAME!!!");
              canClickStartGame.value = false;
            }

            players.value = data.players.map(player => {
              return ({
                name: player.player_name,
                totalCash: player.player_money,
                betCash: player.player_choices?.PlacedInPot?.PlacedInPot || 0,
                faceUpCards: player.player_hand.cards.filter(card => card.face_up).map(card => [formatCard(card), false]),
                faceDownCards: player.player_name === playerName.value || isSpectator.value
                  ? player.player_hand.cards.filter(card => !card.face_up).map(card => [formatCard(card), false])
                  : player.player_hand.cards.filter(card => !card.face_up).map(() => ["", false]),
                lastMove: player.last_move,
                playerAction: player.player_action,
                token: player.token,
            })
          });
        }

        if (data.game_variant) {
          gameVariant.value = data.game_variant;
        }

        if (data.pot) {
          pot.value = data.pot;
        }

        if (data.max_players) {
          maxPlayers.value = data.max_players;
        }

        if (data.players && data.spectators && data.dealer_choice_spectators && data.lobby) {
          spectators.value = data.spectators;
          dealerChoiceSpectators.value = data.dealer_choice_spectators;
          if (
            (!data.players.find(p => p.player_name == playerName.value) &&
            !data.spectators.find(p => p.player_name == playerName.value) &&
            data.lobby.find(p => p.player_name == playerName.value) &&
            data.winner.length == 0) ||
            isDealer.value
          ) {
            canMakeLobbyAction.value = true;
          } else {
            canMakeLobbyAction.value = false;
          }
          // this is the first player - show them the dealer options
          if (data.players.length == 0) {
            const local_data = {
              headerText: "Welcome! You are the first one here! What would you like to do?",
              tableOptions: ["Join table", "Spectate game", "Leave table"],
              dealerOptions: ["Five-Card Draw", "Seven-Card Stud", "Texas Hold 'Em"],
              isDealer: true,
            };
            joinGameMenuData.value = local_data;
          } else if (data.winner.length || data.players.find(p => p.player_name == playerName.value)) {
            showJoinGameMenu.value = false;
          } else { // this is not the first player - show them generic options
            const local_data = {
              headerText: `Welcome! Next table is a ${gameVariant.value} game! What would you like to do?`,
              tableOptions: ["Join table", "Spectate game", "Leave table"],
              dealerOptions: [],
              isDealer: false,
            };
            joinGameMenuData.value = local_data;
          }
        }

        if (data.winner) {
          if (data.winner.length) {
            winners.value = data.winner.map(w => w[0].player_name);
            if (!isDealerChoiceSpectator.value) {
              showEndMenu.value = true;
            }
            gameStarted.value = false;
            if (showEndMenu.value) {
              clickEndMenu();
            }
          } else {
            winners.value = [];
            if (showEndMenu.value) {
              clickEndMenu();
            }
          }
        }

        if (data.current_action_string) {
          currentAction.value = data.current_action_string;
        }

        highestBet.value = data.highest_bet ?? 0;

        if (data.raise_min_max) {
          minRaise.value = data.raise_min_max[0];
          maxRaise.value = data.raise_min_max[1];
        }

        if (data.current_player) {
          currentPlayer.value = data.current_player;
        }

        if (data.discard_cards_prompted !== undefined) {
          discardRound.value = data.discard_cards_prompted;
        }

        if (data.demo_mode !== undefined) {
          demoMode.value = data.demo_mode;
        }
  } catch (e) {
    console.error("WebSocket parse error:", e);
  }
};

const handleLogin = (data) => {
  playerName.value = data.username;
  playerNameEntered.value = true;
  if (players.value.find(p => p.name === data.username)) {
    showJoinGameMenu.value = false;
  } else {
    showJoinGameMenu.value = true;
  }
  if (
    ((!players.value.find(p => p.player_name == playerName.value) &&
    !spectators.value.find(p => p.player_name == playerName.value) &&
    lobby.value.find(p => p.player_name == playerName.value) &&
    winners.value.length == 0) ||
    isDealer.value) &&
    showJoinGameMenu.value == false
  ) {
    canMakeLobbyAction.value = true;
    clickEndMenu();
  } else {
    canMakeLobbyAction.value = false;
  }
};

const resetClientState = () => {
  console.log("RESETTING CLIENT STATE");
  players.value = [];
  communityCards.value = [];
  pot.value = 500;
  highestBet.value = 0;
  playerName.value = "";
  currentPlayer.value = null;
  playerNameEntered.value = false;
  minRaise.value = 0;
  maxRaise.value = 0;
  discardRound.value = false;
  demoMode.value = "";
  winners.value = [];
  gameStarted.value = false;
  canClickStartGame.value = false;
  maxPlayers.value = 0;
  showEndMenu.value = false;
  showJoinGameMenu.value = false;
  endMenuData.value = null;
  joinGameMenuData.value = null;
  seeStats.value = false;
  playerData.value = [];
  gamesData.value = [];
  singleGameData.value = [];
  seeHelp.value = false;
  canMakeLobbyAction.value = false;
  dealerIdx.value = 0;
  gameVariant.value = "";
  spectators.value = [];
  dealerChoiceSpectators.value = [];
  lobby.value = [];
  selectCardsActive.value = true;
  currentAction.value = "Waiting to Start a Game...";
};
</script>

<style scoped lang="postcss">
@reference "tailwindcss";

.app_container {
  @apply flex flex-col justify-center items-center m-8 h-full;

  &.unselectable {
    @apply select-none;
  }

  .action_info {
    @apply text-5xl font-bold max-w-[80%];
  }

  .controls {
    @apply w-full m-4;
  }

  .stats_menu {
    @apply absolute right-8 top-10 border border-black border-solid flex flex-row justify-center items-start bg-gray-300 w-fit overflow-y-auto overflow-x-hidden z-10 max-h-[400px];

    .stats_menu_open {
      @apply flex flex-col justify-center items-center;
    }
  }

  .help_menu {
    @apply absolute left-8 top-10 border border-black border-solid flex flex-row justify-center items-start bg-gray-300 w-fit overflow-y-auto overflow-x-hidden z-10 max-h-[400px];
    .help_menu_open {
      @apply flex flex-col justify-center items-center;
    }
  }

  .control_button {
    @apply border border-black bg-white p-2 select-none text-base;

    &:hover {
      @apply bg-gray-400 cursor-pointer;
    }

    &:active {
      @apply bg-green-500 cursor-pointer;
    }

    &.unavailable {
      @apply cursor-not-allowed bg-gray-300 brightness-75;
    }

    &.selected {
      @apply bg-green-500;
    }
  }
}
.end_menu {
  @apply absolute left-1/2 top-1/2 bg-white border-2 border-solid border-black drop-shadow-md;
  -ms-transform: translate(-50%, -50%);
  transform: translate(-50%, -50%);
}

.join_game_menu {
  @apply absolute left-1/2 top-1/2 bg-white border-2 border-solid border-black drop-shadow-md;
  -ms-transform: translate(-50%, -50%);
  transform: translate(-50%, -50%);
}

.close_btn {
  @apply rounded-md border border-solid border-black w-fit justify-self-end self-end p-2 m-2 cursor-pointer bg-white select-none;
}
</style>
