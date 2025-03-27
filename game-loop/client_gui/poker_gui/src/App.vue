<template>
  <div class="app_container">
    <input
        v-model="playerName"
        v-if="!playerNameEntered"
        type="text" 
        placeholder="Input your name"
        @keyup="(e) => {
          if (e.key === 'Enter' || e.keyCode === 13) {
            playerNameEntered = true;
            joinGame();
          }
        }"
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
    </template>
  </div>
</template>

<script setup>
import Table from './components/Table.vue';
import Controls from './components/Controls.vue';
import { computed, ref, watch } from 'vue';

const players = ref([
  {
    name: "Player1",
    totalCash: 100,
    betCash: 10,
    faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
    faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
    lastMove: "Check",
    token: "D",
  }, {
    name: "Player2",
    totalCash: 100,
    betCash: 10,
    faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
    faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
    lastMove: "Check",
    token: "SB",
  }, {
    name: "Player3",
    totalCash: 100,
    betCash: 10,
    faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
    faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
    lastMove: "Check",
    token: "BB",
  }, {
    name: "Player4",
    totalCash: 100,
    betCash: 10,
    faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
    faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
    lastMove: "Check",
    token: "",
  }
]);

const formatCard = (card) => {
            const valueMap = {
                "Ace": "A", "Two": "2", "Three": "3", "Four": "4", "Five": "5",
                "Six": "6", "Seven": "7", "Eight": "8", "Nine": "9", "Ten": "T",
                "Jack": "J", "Queen": "Q", "King": "K"
            };

            const suitMap = {
                "Heart": "H", "Diamond": "D", "Club": "C", "Spade": "S"
            };

            return `${valueMap[card.value]}${suitMap[card.suit]}`;
        };



const communityCards = ref([["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]]);
const pot = ref(500);
const highestBet = ref(0);
const playerName = ref("");
const playerAction = ref("");
const currentPlayer = ref(null);
const playerNameEntered = ref(false);
const minRaise = ref(0);
const maxRaise = ref(0);
const discardRound = ref(false);

const isCurrentPlayer = computed(() => {
  return currentPlayer.value && currentPlayer.value.player_name === playerName.value;
});


watch(isCurrentPlayer, (newValue) => {
  if (newValue) {
    console.log("✅ You ARE the current player.");
  } else {
    console.log("❌ You are NOT the current player.");
  }
});

const controls = computed(() => {
  const currentPlayer = players.value.find(player => player.name === playerName.value);
  if (!currentPlayer) {
    return [];
  }

  const isCurrentPlayerTurn = isCurrentPlayer.value;

  if (!isCurrentPlayerTurn) {
    return []; 
  }

  return [
    {
      name: "Fold",
      visible: !discardRound.value,  
    },
    {
      name: "Check",
      visible: !discardRound.value && (highestBet.value === 0 || currentPlayer.betCash == highestBet.value),
    },
    {
      name: "Call",
      visible: !discardRound.value && (highestBet.value > 0 && currentPlayer.totalCash >= (highestBet.value - currentPlayer.betCash) && currentPlayer.betCash !== highestBet.value),
    },
    {
      name: "Raise",
      visible: !discardRound.value && currentPlayer.totalCash > 0 && (currentPlayer.totalCash + currentPlayer.betCash) >= minRaise.value,
      needsAmount: true,
      amountMin: minRaise.value,
      amountMax: maxRaise.value, 
    }
  ];
});

// must be set by the server
const selectCardsActive = ref(true);
const currentAction = ref("Player1: Selecting cards to discard...");

const onCardClick = (card) => {
  if (!selectCardsActive.value) {
    return;
  }

const formattedCard = typeof card === "object" ? formatCard(card) : card;

  const player = players.value.find((player) => player.name === playerName.value);
  if (!player) {
    console.error("Player not found:", playerName.value);
    return;
  }

  const clickedCard = player.faceDownCards.find((fdcard) => fdcard[0] === formattedCard);
  if (!clickedCard) {
    console.error("Card not found in player's hand:", formattedCard);
    return;
  }

  clickedCard[1] = !clickedCard[1];
}

const submitOnClick = () => {
  if (!discardRound) { return; }
  
  selectCardsActive.value = false;

  const player = players.value.find((player) => player.name === playerName.value);
  if (!player) {
    console.error("Player not found:", playerName.value);
    return;
  }

  
  const selectedCardIndices = player.faceDownCards
    .map((fdcard, index) => fdcard[1] ? index : -1) 
    .filter(index => index !== -1); 

  const payload = {
    type: "DiscardAction",
    player_name: playerName.value,
    card_index: selectedCardIndices.length === 0 ? [] : selectedCardIndices
  };

  socket.send(JSON.stringify(payload));
  console.log("Sent action:", payload);
}


const onControlsClick = (message) => {
    const { action, betAmount } = message;
    
    const betAmountNumber = Number.isNaN(Number(betAmount)) ? 0 : Math.max(0, betAmount);

    const payload = {
        type: "PlayerAction",
        action,
        player_name: playerName.value,
        bet_amount: betAmountNumber,
    };

    socket.send(JSON.stringify(payload));
    console.log("Sent action:", payload);
};


// Set up WebSocket connection
const socket = new WebSocket("ws://localhost:8080/ws/");
socket.onerror = (error) => console.error("WebSocket error:", error);
socket.onclose = () => console.log("WebSocket connection closed");
// Add to WebSocket open handler
socket.onopen = () => {
    console.log("WebSocket connection opened");
    // Request full state immediately after connection
    fetchGameState();
};

// Modify message handler for better debugging
socket.onmessage = (event) => {
    try {
        const data = JSON.parse(event.data);
        console.log("Received game state:", data);
        
        // Update community cards
        if (data.community_cards?.cards) {
            communityCards.value = data.community_cards.cards.map(card => [card, false]);
        }
       
        // Update players
        if (data.players) {
            
            players.value = data.players.map(player => ({
                name: player.player_name,
                totalCash: player.player_money,
                betCash: player.player_choices?.PlacedInPot?.PlacedInPot || 0,
                faceUpCards: [],
                faceDownCards: player.player_name === playerName.value 
                    ? player.player_hand.cards.map(card => [formatCard(card), false])
                    : player.player_hand.cards.map(() => ["", false]),                
                    lastMove: player.last_move,
                playerAction: player.player_action,
                token: player.token,
            }));
        }
        pot.value = data.pot;
        currentAction.value = data.current_action_string;

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
        
    } catch (e) {
        console.error("WebSocket parse error:", e);
    }
};

// Fallback: fetch game state periodically
async function fetchGameState() {
  // try {
  //   const response = await fetch("http://localhost:3000/game");
  //   const data = await response.json();
  //   console.log("Fetched game state:", data);
  //   const players_hands = data.players ? data.players.map((player) => player.player_hand.cards) : [];
  //   console.log(`Players hands: ${players_hands}`);
  //   renderHands(players_hands);
  // } catch (error) {
  //   console.error("Error fetching game state:", error);
  // }
}

// Render hands from the game state
function renderHands(serverPlayers) {
  // console.log("Rendering players:", players);
  
  if (!serverPlayers || typeof serverPlayers !== "object") {
      console.warn("Invalid players data format");
      return;
  }

  players.value = [];

  Object.entries(serverPlayers).forEach(([name, player]) => {
    let newPlayer = {};
    newPlayer['name'] = name;
    newPlayer['totalCash'] = 100;
    newPlayer['betCash'] = 10;
    newPlayer['faceUpCards'] = [];
    newPlayer['faceDownCards'] = [];
    newPlayer['lastMove'] = "Check";
    if (name === playerName.value) {
        player.hand.forEach(card => {
          newPlayer['faceDownCards'].push([card, false]);
        });
    } else {
        player.hand.forEach(card => {
          newPlayer['faceDownCards'].push(["", false]);
        });
    }
    players.value.push(newPlayer);
  });
}

async function joinGame() {
    console.log(`Joining game as ${playerName.value}`);
    try {
        const response = await fetch(`http://localhost:3000/register/${playerName.value}`, { 
            method: "POST" 
        });
        
        if (!response.ok) {
            const error = await response.text();
            throw new Error(error);
        }
        
        const data = await response.json();
        console.log("Registration response:", data);
        
        // Force initial render from WebSocket
        await new Promise(resolve => setTimeout(resolve, 50));
        await fetchGameState();
        
    } catch (error) {
        console.error("Join error:", error);
        alert(`Join failed: ${error.message}`);
    }
}

async function removeCard(card) {
  try {
    console.log(`Removing card ${card} for player ${playerName.value}`);
    await fetch("http://localhost:3000/remove-card", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ player: playerName.value, card }),
    });
    await fetchGameState();
  } catch (error) {
    console.error("Error removing card:", error);
  }
}

</script>

<style scoped lang="postcss">
@reference "tailwindcss";

.app_container {
  @apply flex flex-col justify-center items-center m-8 h-full;

  .action_info {
    @apply text-5xl font-bold;
  }

  .controls {
    @apply w-full m-4;
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
</style>
