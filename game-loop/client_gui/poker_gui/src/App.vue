<template>
  <div class="app_container" :class="{ unselectable: showEndMenu }">
    <Login
      v-if="!playerNameEntered"
      :playerName="playerName"
      :playerNameEntered="playerNameEntered"
      @loggedIn="handleLogin"
    />
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
    </template>
  </div>
  <div v-if="showEndMenu" class="end_menu">
    <div class="close_btn" @click="closeEndMenu">X</div>
    <EndRoundScreen v-bind="endMenuData" @click="submitEndRound" />
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

const players = ref([]);
const communityCards = ref([
  ["AH", false],
  ["2D", false],
  ["TC", false],
  ["QS", false],
  ["7C", false],
]);
const pot = ref(500);
const highestBet = ref(0);
const playerName = ref("");
const playerAction = ref("");
const currentPlayer = ref(null);
const playerNameEntered = ref(false);
const minRaise = ref(0);
const maxRaise = ref(0);
const discardRound = ref(false);
const demoMode = ref("");
const winners = ref([]);
const showEndMenu = ref(false);
const endMenuData = ref(null);
const seeStats = ref(false);
const playerData = ref([]);
const gamesData = ref([]);
const singleGameData = ref([]);
const seeHelp = ref(false);

// must be set by the server
const selectCardsActive = ref(true);
const currentAction = ref("Waiting to Start a Game...");

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

watch(isCurrentPlayer, (newValue) => {
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

const clickEndMenu = () => {
  const dealerName = players.value.find((player) => player.token === "D")
    ? players.value.find((player) => player.token === "D").name
    : "";
  const data = {
    headerText: winnersText.value,
    tableOptions: ["Play again", "Leave table"],
    dealerOptions: ["Five-Card Draw", "Seven-Card Stud", "Texas Hold 'Em"],
    isDealer: dealerName == playerName.value,
  };
  showEndMenu.value = true;
  endMenuData.value = data;
};

async function clickStatsMenu(message) {
  if (seeStats.value) {
    const payload = {
      type: "StatsMenu",
      ...message,
    };
    console.log("Send stats option:", JSON.stringify(payload));

    try {
      const response = await fetch(
        `http://localhost:3000/stats?type=${payload.type}&stats_menu_type=${payload.stats_menu_type}&selected_option=${payload.selected_option}`,
        {
          method: "GET",
        }
      );

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error);
      }

      const data = await response.json();
      console.log("Stats get response:", data);

      if (data.player_data) {
        console.log("player data: ", data.player_data);
        const inner_data = JSON.parse(data.player_data);
        console.log("inner_data: ", inner_data);
        playerData.value = inner_data;
      }

      if (data.games_data) {
        console.log("games data: ", data.games_data);
        const inner_data = JSON.parse(data.games_data);
        console.log("inner_data: ", inner_data);
        gamesData.value = inner_data;
      }

      if (data.single_game_data) {
        console.log("single game data: ", data.single_game_data);
        const inner_data = JSON.parse(data.single_game_data);
        console.log("inner_data: ", inner_data);
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
  const payload = {
    type: "EndRound",
    ...message,
  };
  socket.send(JSON.stringify(payload));
  console.log("Send end option:", payload);
  showEndMenu.value = false;
  winners.value = [];
};

// Set up WebSocket connection
const socket = new WebSocket(`ws://localhost:8080/ws/`);
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
    console.log("Received game state:", data);

    // Update community cards
    if (data.community_cards?.cards) {
      communityCards.value = data.community_cards.cards.map((card) => [
        formatCard(card),
        false,
      ]);
    }

    // Update players
    if (data.players) {
      players.value = data.players.map((player) => ({
        name: player.player_name,
        totalCash: player.player_money,
        betCash: player.player_choices?.PlacedInPot?.PlacedInPot || 0,
        faceUpCards: player.player_hand.cards
          .filter((card) => card.face_up)
          .map((card) => [formatCard(card), false]),
        faceDownCards:
          player.player_name === playerName.value
            ? player.player_hand.cards
                .filter((card) => !card.face_up)
                .map((card) => [formatCard(card), false])
            : player.player_hand.cards
                .filter((card) => !card.face_up)
                .map(() => ["", false]),
        lastMove: player.last_move,
        playerAction: player.player_action,
        token: player.token,
      }));
    }

    if (data.pot) {
      pot.value = data.pot;
    }

    if (data.winner) {
      if (data.winner.length) {
        winners.value = data.winner.map((w) => w[0].player_name);
        showEndMenu.value = true;
        clickEndMenu();
      } else {
        winners.value = [];
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
    @apply text-5xl font-bold;
  }

  .controls {
    @apply w-full m-4;
  }

  .stats_menu {
    @apply absolute right-8 top-10 border border-black border-solid flex flex-row justify-center items-center bg-gray-300 w-fit overflow-y-auto overflow-x-hidden;

    .stats_menu_open {
      @apply flex flex-col justify-center items-center;
    }
  }

  .help_menu {
    @apply absolute left-8 top-10 border border-black border-solid flex flex-row justify-center items-center bg-gray-300 w-fit overflow-y-auto overflow-x-hidden;
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
.close_btn {
  @apply rounded-md border border-solid border-black w-fit justify-self-end self-end p-2 m-2 cursor-pointer bg-white select-none;
}
</style>
