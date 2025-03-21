<template>
  <div
    class="card_container"
    :class="{selected}"
    @click="onClick"
  >
    <div class="card_top">
      <span :class=cardColor>{{ rank }}</span>
      <span :class=cardColor>{{ suit }}</span>
    </div>
    <div class="card_middle">
      <span :class=cardColor>{{ rank }}</span>
    </div>
    <div class="card_bottom">
      <span :class=cardColor>{{ rank }}</span>
      <span :class=cardColor>{{ suit }}</span>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  rank: String,
  suit: String,
  selected: Boolean,
});

const rank = computed(() => props.rank ?? "#");
const suit = computed(() => props.suit ?? "#");

const cardColor = computed(() => {
  // Determine the card color based on the suit (Red suits are hearts and diamonds)
  if (props.suit === "H" || props.suit === "D") {
    return "red";
  }
  return "black"; // Spades and Clubs are black
});

const emit = defineEmits(['click'])

const onClick = () => {
    if (!props.disabled) {
        emit('click', rank.value+suit.value);
    }
};
</script>

<style scoped lang="postcss">
  @reference "tailwindcss";
.card_container {
  @apply flex flex-col w-16 h-fit bg-white border-2 border-gray-400 rounded-lg shadow-lg overflow-hidden select-none;

  &.selected {
    @apply bg-green-700;
  }

  .card_top {
    @apply flex p-1 text-base;
  }
  .card_middle {
    @apply flex justify-center items-center h-full text-lg;
  }
  .card_bottom {
    @apply flex p-1 text-base justify-end;
  }
}

.red {
  @apply text-red-600;
}
.black {
  @apply text-black;
}
</style>
