<template>
    <div class="controls_container">
        <div class="buttons">
            <div
                class="control_button"
                :class="{
                    unavailable: !control.visible,
                    selected: selectedControlName == control.name,
                }"
                v-for="control in controls"
                @click="controlOnClick(control)"
            >
                {{ control.name }}
            </div>
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
        <div class="inputs">
            <input
                v-model="inputText"
                v-if="inputsVisible"
                class="control_input"
                type="text" 
                :placeholder="placeholderText"
            />
        </div>
    </div>
</template>

<script setup>
import { computed, ref } from 'vue';

const props = defineProps({
  /** @type {{
      name: String,
      needsAmount: Boolean,
      amountMin: Number,
      amountMax: Number,
      visible: Boolean,
    }[]} */
    controls: Array,
});

const canSubmit = ref(false);
const inputsVisible = ref(false);
const selectedControlName = ref(null);
const inputText = ref("");
const amountMin = ref(0);
const amountMax = ref(0);

const placeholderText = computed(() => `Enter amount between ${amountMin.value} and ${amountMax.value}...`);

const setInputVis = () => {
    inputsVisible.value = true;
};

const resetInputVis = () => {
    inputsVisible.value = false;
};

const disableAllControlVisibilities = () => {
    props.controls.forEach((control) => control.visible = false);
}

const controlOnClick = (control) => {
    if (!control.visible) { return; }
    if (control.needsAmount) {
        setInputVis();
    } else {
        resetInputVis();
    }
    canSubmit.value = true;
    selectedControlName.value = control.name;
    amountMin.value = control.amountMin ?? 0;
    amountMax.value = control.amountMax ?? 0;
}

const submitOnClick = () => {
    if (!canSubmit.value) { return; }
    canSubmit.value = false;
    amountMin.value = 0;
    amountMax.value = 0;
    inputsVisible.value = false;
    disableAllControlVisibilities();
    onClick();
}

const emit = defineEmits(['click']);

const onClick = () => {
    if (!props.disabled) {
        emit('click', [selectedControlName.value, inputText.value]);
        selectedControlName.value = "";
    }
};

</script>

<style scoped lang="postcss">
@reference "tailwindcss";
.controls_container {
    @apply flex flex-col gap-4 justify-center items-center;

    .buttons {
        @apply flex flex-row gap-4 text-base;

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

    .inputs {
        @apply flex justify-center w-full;

        .control_input {
            @apply p-4 text-xs font-semibold w-[50%] overflow-ellipsis;
        }
    }
}
</style>
