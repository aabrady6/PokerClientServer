<template>
  <div class="table">
    <div class="community-area">
      <div
        v-if="communityCardsFormatted.length"
        class="community-cards"
      >
        Community Cards:
        <div class="cards">
          <Card
            v-for="card in communityCardsFormatted"
            :rank="card.rank"
            :suit="card.suit"
            :selected="false"
          />
        </div>
      </div>
      <div class="pot">Pot: ${{ pot }}</div>
    </div>

    <span class="cards_label">Other Cards:</span>
    <!-- Player Mats positioned around the table -->
    <div class="players">
      <div class="other_players_container">
        <PlayerMat
          class="player_mat"
          v-for="player in otherPlayers"
          :player="player"
          :token="player.token"
          :type="'overlap'"
        />
      </div>
    </div>

    <span class="cards_label">Your Cards:</span>
    <div class="this_player">
      <PlayerMat
        v-if="thisPlayer"
        class="player_mat"
        :player="thisPlayer"
        :token="thisPlayer.token"
        :type="'full'"
        @click="onClick"
      />
    </div>
  </div>
</template>

<script setup>
import { computed, watch } from "vue";
import PlayerMat from "./PlayerMat.vue";
import Card from "./Card.vue";

const props = defineProps({
  /** @type {{
      name: String,
      totalCash: Number,
      betCash: Number,
      faceDownCards: [String, Boolean][],
      faceUpCards: [String, Boolean][],
      lastMove: String,
      token: String,
    }[]} */
  players: Array,
  /** @type { [String, Boolean][] } */
  communityCards: Array,
  pot: Number,
  playerName: String,
});

const thisPlayer = computed(() => props.players.find((player) => player.name == props.playerName));

const otherPlayers = computed(() => props.players.filter((player) => player.name != props.playerName));

const communityCardsFormatted = computed(() => props.communityCards.map((card) => ({ rank: card[0][0], suit: card[0][1] })));

const emit = defineEmits(['click'])

const onClick = (card) => {
    if (!props.disabled) {
        emit('click', card);
    }
};

</script>

<style scoped lang="postcss">
@reference "tailwindcss";

.table {
  @apply flex flex-col w-full h-full m-5 p-4 border-4 border-gray-600 bg-green-700 shadow-inner;

  .community-area {
    @apply flex flex-col items-center gap-4 text-lg font-bold;
    
    .community-cards {
      @apply flex flex-col justify-center items-center;

      .cards {
        @apply flex justify-center gap-2 mb-2;
      }
    }


    .pot {
      @apply text-2xl font-bold text-yellow-500;
    }
  }

  .cards_label {
    @apply flex justify-center items-center text-4xl font-bold;
  }

  .this_player {
    @apply flex justify-center items-center text-white text-xl font-bold;
  }

  .players {
    @apply flex flex-row px-8 justify-center;

    .other_players_container {
      @apply flex flex-row gap-8 text-xl text-white font-bold overflow-auto;
    }
  }
}
</style>
