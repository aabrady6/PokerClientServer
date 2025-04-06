<template>
    <div class="end_round_container">
        <div class="header_text">{{ headerText }}</div>
        <div class="table_options">
            <div
                v-for="option in tableOptions"
                class="control_button"
                :class="{
                    selected: selectedTableOption === option,
                }"
                @click="tableOptionSelect(option)"
            >
                {{ option }}
            </div>
        </div>
        <ChooseGameType
            v-if="isDealer && selectedTableOption === tableOptions[0]"
            :dealerOptions="dealerOptions"
            @click="dealerOptionSelect"
        />
        <div
            class="control_button"
            :class="{
                unavailable: !canSubmit
            }"
            @click="submitOnClick"
        >
            SUBMIT
        </div>
    </div>
</template>

<script setup>
import { ref } from 'vue';
import ChooseGameType from './ChooseGameType.vue';

const props = defineProps({
    headerText: String,
    tableOptions: Array[String],
    dealerOptions: Array[String],
    isDealer: Boolean,
});

const canSubmit = ref(false);
const selectedTableOption = ref("");
const selectedDealerOption = ref("");

const tableOptionSelect = (option) => {
    selectedTableOption.value = option;
    if (!props.isDealer || option != props.tableOptions[0]) {
        canSubmit.value = true;
    } else {
        canSubmit.value = false;
    }
}

const dealerOptionSelect = (option) => {
    selectedDealerOption.value = option;
    canSubmit.value = true;
}

const submitOnClick = () => {
    if (!canSubmit.value) { return; }
    canSubmit.value = false;
    onClick();
}

const emit = defineEmits(['click']);

const onClick = () => {
    if (!props.disabled) {
        // TODO: add in emit
        let message = {};

        if (props.isDealer) {
            message = {
                option: selectedTableOption.value,
                dealer_option: selectedDealerOption.value,
            };
        } else {
            message = {
                option: selectedTableOption.value,
                dealer_option: "",
            };
        }

        emit('click', message);
        selectedTableOption.value = "";
        selectedDealerOption.value = "";
    }
};

</script>

<style scoped lang="postcss">
@reference "tailwindcss";
.end_round_container {
    @apply flex flex-col p-4 gap-4 justify-center items-center text-center;

    .header_text {
        @apply text-4xl;
    }

    .table_options {
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
}
</style>
