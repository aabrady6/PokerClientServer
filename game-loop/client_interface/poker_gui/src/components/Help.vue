<template>
  <div class="help_container">
    <div
      v-if="helpMenuType !== 'root'"
      class="control_button back_btn"
      @click="onBack"
    >
      BACK
    </div>

    <div v-if="helpMenuType === 'root'" class="help_options">
      <div class="control_button" @click="onClick('five_card_draw')">
        5 Card Draw
      </div>
      <div class="control_button" @click="onClick('seven_card_stud')">
        7 Card Stud
      </div>
      <div class="control_button" @click="onClick('texas_holdem')">
        Texas Hold'em
      </div>
    </div>

    <div v-if="helpMenuType !== 'root'" class="help_text">
      <p v-html="selectedHelpText"></p>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";

const helpMenuType = ref("root");
const selectedHelpText = ref("");

const helpTexts = {
  five_card_draw: `
  <b><c><h3>Five Card Draw</b></c></h3><br>
  <b>House Rules:</b>
  <ul>
    <li>Players must match all bets, if someone is unable to cover the full bet amount, they must fold</li>
    <li>If a player runs out of money, between rounds they are refilled to $1000</li>
  </ul>
  <b>Basic Game Rules:</b><br>
  1. Pre-game actions (betting, blind wagers)
  <ul>
    <li>Players must wager the blinds to begin playing. Small blind: $2, Big blind: $5. This is done automatically.</li>
    <li>The player to the left of the dealer is the small blind.</li>
    <li>The player to the left of the small blind is the big blind.</li>
    <li>All other players do not wager intially.</li>
  </ul>
  2. The dealer deals 5 face down cards to each player.<br>
  3. Initial betting round:
  <ul>
  <li>Control the player with the control buttons on the bottom of the screen.</li>
  <ul>
    <li>Demo Mode button calls all players to the highest bet, and then skips to the last betting round.</li>
  </ul>
  <li>The player to the left of the small blind begins bettig. If that player has folded, then the player to thier left starts.</li>
  <li>Players must match the initial highest bet before moving to the next round.</li>
  <li>If no other player raised during the betting round, the big blind player gets the opportunity to bet.</li>
  </ul>
  <br> 4. Discard Round:
  <ul>
    <li>Player to the left of the big blind begins the discard round.</li>
    <li>Players select cards to discard by clicking on each card and pressing submit.</li>
    <li>Players can discard 0 - 5 of thier cards.</li>
  </ul>
  <br> 5. Final Betting Round:
  <ul>
    <li>Player left of the big blind starts betting round.</li>
    <li>Same betting rules apply as initial betting round.</li>
  </ul>
  <br> 6. Scoring Round:
  <ul>
  <li>The remaining players hands are scored and the pot is payed out to the winner(s).</li>
  </ul>
  <br> 7. End of Game:
  <ul>
  <li>The dealer button rotates, and the new dealer chooses the next game type.</li>
  <li>Players are given the option to play again or leave the table.</li>
  </ul>
  `,
  seven_card_stud: `
  <b><c><h3>Seven Card Stud</b></c></h3><br>
  <b>House Rules:</b>
  <ul>
    <li>Players must match all bets, if someone is unable to cover the full bet amount, they must fold</li>
    <li>If a player runs out of money, between rounds they are refilled to $1000</li>
  </ul>
  <b>Basic Game Rules:</b><br>
  1. Pre-game actions (betting, blind wagers)
  <ul>
    <li>All players must wager $2 ante. This is done automatically.</li>
  </ul>
  2. The dealer deals 2 face down cards and 1 face up card to each player.<br>
  3. Initial betting round:
  <ul>
  <li>The player with the lowest card showing must wager $5 to bring in the round. They start the betting round.</li>
  <li>All players must match the highest bet or fold.</li>
  <li>Control the player with the control buttons on the bottom of the screen.</li>
    <ul>
      <li>Demo Mode button calls all players to the highest bet, and then skips to the last betting round.</li>
    </ul>
  </ul>
  <br> 4. Deal 1 face up card (each player has 2 face down, and 2 face up cards):
  <ul>
    <li>The player with the highest face up poker hand begins the betting round.</li>
  </ul>
  <br> 5. Deal 1 face up card (each player has 2 face down, and 3 face up cards):
  <ul>
    <li>The player with the highest face up poker hand begins the betting round.</li>
  </ul>
  <br> 6. Deal 1 face up card (each player has 2 face down, and 4 face up cards):
  <ul>
    <li>The player with the highest face up poker hand begins the betting round.</li>
  </ul>
  <br> 7. Final Round. Deal 1 face down card (each player has 3 face down, and 4 face up cards):
  <ul>
    <li>The player with the highest face up poker hand begins the betting round.</li>
  </ul>
  <br> 8. Scoring Round:
  <ul>
  <li>The remaining players hands create the best 5 card combination from the cards they own.</li>
  <li>The pot is payed out to the winner(s).</li>
  </ul>
  <br> 9. End of Game:
  <ul>
  <li>The dealer button rotates, and the new dealer chooses the next game type.</li>
  <li>Players are given the option to play again or leave the table.</li>
  </ul>
  `,

  texas_holdem: `
  <b><c><h3>Texas Hold'em</b></c></h3><br>
  <b>House Rules:</b>
  <ul>
    <li>Players must match all bets, if someone is unable to cover the full bet amount, they must fold</li>
    <li>If a player runs out of money, between rounds they are refilled to $1000</li>
  </ul>
  <b>Basic Game Rules:</b><br>
  1. Pre-game actions (betting, blind wagers)
  <ul>
    <li>Players must wager the blinds to begin playing. Small blind: $2, Big blind: $5. This is done automatically.</li>
    <li>The player to the left of the dealer is the small blind.</li>
    <li>The player to the left of the small blind is the big blind.</li>
    <li>All other players do not wager intially.</li>
  </ul>
  2. The dealer deals 2 face down cards to each player.<br>
  3. Initial betting round:
  <ul>
  <li>Control the player with the control buttons on the bottom of the screen.</li>
  <ul>
    <li>Demo Mode button calls all players to the highest bet, and then skips to the last betting round.</li>
  </ul>
  <li>The player to the left of the small blind begins bettig. If that player has folded, then the player to thier left starts.</li>
  <li>Players must match the initial highest bet before moving to the next round.</li>
  <li>If no other player raised during the betting round, the big blind player gets the opportunity to bet.</li>
  </ul>
  <br> 4. 3 community cards are dealt face up.
  <ul> 
    <li>The player to the left of the big blind begins betting.</li>
  </ul>
  <br> 5. 1 community cards are dealt face up (4 community cards).
  <ul> 
    <li>The player to the left of the big blind begins betting.</li>
  </ul>
  <br> 6. 1 community cards are dealt face up (5 community cards).
  <ul> 
    <li>The player to the left of the big blind begins betting.</li>
  </ul>
  <br> 5. Final Betting Round:
  <ul>
    <li>Player left of the big blind starts betting round.</li>
    <li>Same betting rules apply as initial betting round.</li>
  </ul>
  <br> 6. Scoring Round:
  <ul>
  <li>The remaining players hands create the best 5 card combination from the cards they own.</li>
  <li>The pot is payed out to the winner(s).</li>
  </ul>
  <br> 7. End of Game:
  <ul>
  <li>The dealer button rotates, and the new dealer chooses the next game type.</li>
  <li>Players are given the option to play again or leave the table.</li>
  </ul>
  `,
};

const onClick = (option) => {
  helpMenuType.value = option;
  selectedHelpText.value = helpTexts[option];
};

const onBack = () => {
  helpMenuType.value = "root";
  selectedHelpText.value = "";
};
</script>

<style scoped lang="postcss">
@reference "tailwindcss";

.help_container {
  @apply flex flex-col p-4 gap-4 justify-center items-center w-full h-full;
}

.help_options {
  @apply flex flex-col gap-2;
}

.help_text {
  @apply text-lg border border-black p-4 bg-gray-100 rounded-md;
}

.control_button {
  @apply border border-black bg-white p-2 select-none;

  &:hover {
    @apply bg-gray-400 cursor-pointer;
  }

  &:active {
    @apply bg-green-500 cursor-pointer;
  }
}

.back_btn {
  @apply absolute top-4 left-4;
}
</style>
