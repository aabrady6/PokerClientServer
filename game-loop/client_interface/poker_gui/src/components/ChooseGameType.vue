<template>
    <div class="dealer_options">
        <div
            v-for="option in dealerOptions"
            class="control_button"
            :class="{
                selected: selectedDealerOption === option,
            }"
            @click="dealerOptionSelect(option)"
        >
            {{ option }}
        </div>
    </div>
</template>

<script setup>
import { ref } from 'vue';

const props = defineProps({
    dealerOptions: Array[String],
});

const selectedDealerOption = ref("");

const dealerOptionSelect = (option) => {
    selectedDealerOption.value = option;
    emit('click', option);
}

const emit = defineEmits(['click']);

</script>

<style scoped lang="postcss">
@reference "tailwindcss";
.dealer_options {
    @apply flex flex-row gap-2 justify-center items-center;
}

.control_button {
    @apply border border-black bg-white p-2 select-none;

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
</style>
