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
        class="controls"
        :controls="controls"
        @click="onControlsClick"
      />
      <div
        v-if="selectCardsActive"
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
import { computed, ref } from 'vue';

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

const communityCards = ref([["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]]);
const pot = ref(500);
const playerName = ref("");
const playerNameEntered = ref(false);

const controls = ref([{
  name: "Fold",
  visible: true,
}, {
  name: "Call",
  visible: true,
}, {
  name: "Raise",
  visible: true,
  needsAmount: true,
  amountMin: 5,
  amountMax: 1000,
}]);

// must be set by the server
const selectCardsActive = ref(true);
const currentAction = ref("Player1: Selecting cards to discard...");

const onCardClick = (card) => {
  if (!selectCardsActive.value) {
    return;
  }
  const clickedCard = players.value.find((player) => player.name == playerName.value).faceDownCards.find((fdcard) => fdcard[0] == card);
  clickedCard[1] = !clickedCard[1];
}

const submitOnClick = () => {
  if (!selectCardsActive.value) { return; }
  selectCardsActive.value = false;

  // Call function to send card information
  // console.log(players.value.find((player) => player.name == playerName.value).faceDownCards.filter((fdcard) => fdcard[1] == true).map((fdcard) => fdcard[0]));

  const cardsToBeRemoved = players.value.find((player) => player.name == playerName.value).faceDownCards.filter((fdcard) => fdcard[1] == true).map((fdcard) => fdcard[0]);
  cardsToBeRemoved.forEach(card => removeCard(card))
}

const onControlsClick = (action) => {
  // Call function to send action information
  console.log(action);
}

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
    console.log("Raw WebSocket message:", event.data);
    try {
        const data = JSON.parse(event.data);
        console.log("Parsed WebSocket data:", data);
        if (data.players) {
            renderHands(data.players);
        } else {
            console.warn("WebSocket message missing players field");
            fetchGameState(); // Fallback to HTTP request
        }
    } catch (e) {
        console.error("WebSocket parse error:", e);
    }
};

// Fallback: fetch game state periodically
async function fetchGameState() {
  try {
    const response = await fetch("http://localhost:3000/game");
    const data = await response.json();
    console.log("Fetched game state:", data);
    renderHands(data.players);
  } catch (error) {
    console.error("Error fetching game state:", error);
  }
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

let isResetting = false;

async function resetHand() {
    if (isResetting) return;
    isResetting = true;
    
    console.log("Reset Hand button clicked, playerName:", playerName.value);
    try {
        const response = await fetch("http://localhost:3000/reset-hand", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ player: playerName.value }),
        });
        
        console.log("Reset response status:", response.status);
        if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to reset hand");
        }
        
        const json = await response.json();
        console.log("Reset response data:", json);
        await fetchGameState();
    } catch (error) {
        console.error("Error resetting hand:", error);
        alert(error.message);
    } finally {
        isResetting = false;
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
