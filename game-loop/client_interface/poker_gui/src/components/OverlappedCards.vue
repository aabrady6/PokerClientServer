<template>
  <div class="overlapped_container"  :style="containerStyle">
    <Card
        class="card_inst"
        v-for="(card, i) in faceDownCards"
        :rank="card.rank"
        :suit="card.suit"
        :selected="card.selected"
        :style="{
            left: `${i * overlapOffset}px`,
        }"
    />
  </div>
</template>

<script setup>
import { computed } from 'vue';
import Card from './Card.vue';

const props = defineProps({
  /** @type {{
      rank: String,
      suit: String,
      faceUp: Boolean,
      selected: Boolean,
    }[]} */
    cards: Array
});

const faceDownCards = computed(() => props.cards.filter((card) => !card.faceUp));

const cardWidth = 68; //px, defined in Card.vue
const cardHeight = 96; // determined experimentally
const overlapOffset = 30;

const containerStyle = computed(() => ({
  "--card-width": `${cardWidth}px`,
  "--card-height": `${cardHeight}px`,
  "--overlap-offset": `${overlapOffset}px`,
  "--num-cards": faceDownCards.value.length,
}));
</script>

<style scoped lang="postcss">
  @reference "tailwindcss";
.overlapped_container {
  @apply flex flex-row relative;
  width: calc(var(--card-width) + ((var(--num-cards) - 1) * var(--overlap-offset)));
  height: var(--card-height);

  .card_inst {
    @apply absolute;
  }

}
</style>
