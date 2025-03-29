<template>
    <div class="stats_container">
        <div
            v-if="statsMenuType != 'root'"
            class="control_button back_btn"
            @click="onBack"
        >
            BACK
        </div>
        <div
            v-if="statsMenuType === 'root'"
            class="stats_options"
        >
            <div
                class="control_button"
                @click="onClick('PlayerStats')"
            >
                Player stats
            </div>
            <div
                class="control_button"
                @click="onClick('GameStats')"
            >
                Game stats
            </div>
        </div>
        <table
            v-if="statsMenuType === 'player'"
            class="player_stats"
        >
            <thead>
                <tr>
                    <th
                        v-for="(_, key) in playerData[0]"
                        :key="key"
                        class="player_stats_cell"
                    >
                        {{ key }}
                    </th>
                </tr>
            </thead>
            <tbody>
                <tr
                    v-for="data in playerData"
                    :key="data.player_id"
                    class="control_button player_stats_row"
                >
                    <td
                        v-for="(value, key) in data"
                        :key="key"
                        class="player_stats_cell"
                    >
                        {{ value }}
                    </td>
                </tr>
            </tbody>
        </table>
        <table
            v-if="statsMenuType === 'games'"
            class="games_stats"
        >
            <thead>
                <tr>
                    <th
                        v-for="(_, key) in gamesData[0]"
                        :key="key"
                        class="games_stats_cell"
                    >
                        {{ key }}
                    </th>
                </tr>
            </thead>
            <tbody>
                <tr
                    v-for="data in gamesData"
                    :key="data.game_id"
                    class="control_button games_stats_row"
                    @click="onClick(data.game_id)"
                >
                    <td
                        v-for="(value, key) in data"
                        :key="key"
                        class="games_stats_cell"
                    >
                        {{ value }}
                    </td>
                </tr>
            </tbody>
        </table>
        <table
            v-if="statsMenuType == 'single_game'"
            class="single_game_stats"
        >
            <thead>
                <tr>
                    <th
                        v-for="(_, key) in singleGameData[0]"
                        :key="key"
                        class="single_game_stats_cell"
                    >
                        {{ key }}
                    </th>
                </tr>
            </thead>
            <tbody>
                <tr
                    v-for="data in singleGameData"
                    :key="data.player_id"
                    class="control_button single_game_stats_row"
                >
                    <td
                        v-for="(value, key) in data"
                        :key="key"
                        class="single_game_stats_cell"
                    >
                        {{ value }}
                    </td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';

const props = defineProps({
  /** @type {{
      player_id: Number,
      username: String,
      money: Number,
      hands_played: Number,
      win_percentage: String,
      wins: Number,
      losses: Number,
      total_earnings: Number,
    }[]} */
    playerData: Array,
  /** @type {{
      game_id: Number,
      game_variant: String,
      num_players: Number,
      winners: String[],
      dealer_idx: Number,
    }[]} */
    gamesData: Array,
  /** @type {{
      player_id: Number,
      player_name: String,
      hand: String[],
      total_wagered: Number,
      chips_won: Number,
    }[]} */
    singleGameData: Array,
});

const statsMenuType = computed(() => {
    if (props.singleGameData.length) {
        return "single_game";
    } else if (props.gamesData.length) {
        return "games";
    } else if (props.playerData.length) {
        return "player";
    } else {
        return "root";
    }
});

const emit = defineEmits(['click', 'back']);

const onClick = (option) => {
    if (!props.disabled) {
        // TODO: add in emit
        let message = {};

        if (statsMenuType.value === "root") {
            if (option === "PlayerStats") {
                message = {
                    stats_menu_type: "player",
                    selected_option: "",
                };
            } else {
                message = {
                    stats_menu_type: "games",
                    selected_option: "",
                };
            }
        } else if (statsMenuType.value === "games") {
            message = {
                stats_menu_type: "single_game",
                selected_option: option,
            };
        } else {
            message = {
                stats_menu_type: statsMenuType.value,
                selected_option: "",
            };
        }

        emit('click', message);
    }
};

const onBack = () => {
    emit('back', statsMenuType.value);
}

</script>

<style scoped lang="postcss">
@reference "tailwindcss";
.stats_container {
    @apply flex flex-col p-4 gap-4 justify-center items-center w-full h-full;
    
    .stats_options {
        @apply flex flex-col gap-2;
    }

    .player_stats {
        @apply border-collapse;

        .player_stats_row {
            @apply border border-black border-solid;
        }

        .player_stats_cell {
            @apply border border-black border-solid p-1 text-center;
        }
    }

    .games_stats {
        @apply border border-black border-solid border-collapse;

        .games_stats_row {
            @apply border border-black border-solid;
        }

        .games_stats_cell {
            @apply border border-black border-solid p-1 text-center;
        }
    }

    .single_game_stats {
        @apply border border-black border-solid border-collapse;

        .single_game_stats_row {
            @apply border border-black border-solid;
        }

        .single_game_stats_cell {
            @apply border border-black border-solid p-1 text-center;
        }
    }
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
