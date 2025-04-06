<template>
  <div class="player_mat_container">
    <div class="top_row">
      <div class="player_pot">
        Bet: ${{ player.betCash }}
      </div>
      
      <div
          v-if="!!token"
          class="token"
        >
          <img :src="token_url" class="token_img" />
      </div>
    </div>
    <div class="second_row">
      <div class="last_move">Last move: {{ player.lastMove }}</div>
      <div class="total_cash">Total: ${{ player.totalCash }}</div>
    </div>
    <div
      v-if="upCardsProper.length"
      class="face_up"
    >
      <Card
        v-if="type == 'full'"
        v-for="card in upCardsProper"
        :rank="card.rank"
        :suit="card.suit"
        :selected="card.selected"
      />
      <OverlappedCards v-else :cards="upCardsProper" />
    </div>
    <div 
      v-if="downCardsProper.length"
      class="face_down"
    >
      <Card
        v-if="type == 'full'"
        v-for="card in downCardsProper"
        :rank="card.rank"
        :suit="card.suit"
        :selected="card.selected"
        @click="onClick"
      />
      <OverlappedCards v-else :cards="downCardsProper" />
    </div>
    <div class="player_name">
      {{ player.name }}
    </div>
  </div>
</template>

<script setup>
import { computed } from "vue";
import Card from "./Card.vue";
import OverlappedCards from "./OverlappedCards.vue";

const props = defineProps({
  /** @type {{
      name: String,
      totalCash: Number,
      betCash: Number,
      faceDownCards: [String, Boolean][],
      faceUpCards: [String, Boolean][],
      lastMove: String,
    }} */
  player: {
    name: String,
    totalCash: Number,
    betCash: Number,
    faceDownCards: Array[[String, Boolean]],
    faceUpCards: Array[[String, Boolean]],
    lastMove: String,
  },
  type: String,
  token: String,
});

const upCardsProper = computed(() => {
  return (props.player.faceUpCards.length
    ? props.player.faceUpCards.map((card) => ({ rank: card[0][0], suit: card[0][1], selected: card[1] }))
    : []);
});

const downCardsProper = computed(() => {
  return (props.player.faceDownCards.length
    ? props.player.faceDownCards.map((card) => ({ rank: card[0][0], suit: card[0][1], selected: card[1] }))
    : []);
});

const token_url = computed(() => {
  if (props.token == 'D') {
    return "./src/images/DealerToken.png";
  } else if (props.token == 'SB') {
    return "./src/images/SmallBlindToken.png";
  } else if (props.token == 'BB') {
    return "./src/images/BigBlindToken.png";
  } else {
    return "";
  }
});

const emit = defineEmits(['click'])

const onClick = (card) => {
    if (!props.disabled) {
        emit('click', card);
    }
};
</script>

<style scoped lang="postcss">
@reference "tailwindcss";

.player_mat_container {
  @apply flex flex-col w-fit h-fit gap-4 justify-center items-center;

  .top_row {
    @apply flex flex-row items-center;
  
    .player_pot {
      @apply text-4xl text-yellow-500;
    }

    .token {
      @apply relative left-7 w-10 h-10;

      .token_img {
        @apply w-full h-full;
      }
    }

  }

  .second_row {
    @apply flex flex-row gap-4 items-center;
  }

  .face_up {
    @apply flex flex-row gap-2 p-4 bg-green-300;
  }

  .face_down {
    @apply flex flex-row gap-2 p-4 bg-blue-300;
  }

  .player_name {
    @apply text-lg;
  }
}
</style>
