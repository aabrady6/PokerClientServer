import { action } from '@storybook/addon-actions';

import Stats from '@/components/Stats.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/Stats',
  component: Stats,
  tags: ['autodocs'],
  render: (args) => ({
    components: { Stats },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<Stats v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    playerData: [],
    gamesData: [],
    singleGameData: [],
  },
};

export const PlayerData = {
  args: {
    playerData: [
      {
        "player_id": 17,
        "username": "fake_gene",
        "money": 1000,
        "hands_played": 10,
        "win_percentage": "50.00%",
        "wins": 5,
        "losses": 5,
        "total_earnings": 100
      },
      {
        "player_id": 18,
        "username": "fake_frank",
        "money": 2000,
        "hands_played": 100,
        "win_percentage": "20.00%",
        "wins": 20,
        "losses": 80,
        "total_earnings": 200
      },
      {
        "player_id": 19,
        "username": "fake_alice",
        "money": 3000,
        "hands_played": 1000,
        "win_percentage": "80.00%",
        "wins": 800,
        "losses": 200,
        "total_earnings": 300
      },
      {
        "player_id": 20,
        "username": "fake_brian",
        "money": 4000,
        "hands_played": 20,
        "win_percentage": "0.00%",
        "wins": 0,
        "losses": 20,
        "total_earnings": 400
      },
      {
        "player_id": 21,
        "username": "fake_sam",
        "money": 5000,
        "hands_played": 200,
        "win_percentage": "100.00%",
        "wins": 200,
        "losses": 0,
        "total_earnings": 500
      },
      {
        "player_id": 22,
        "username": "fake_wanda",
        "money": 5995,
        "hands_played": 34,
        "win_percentage": "32.35%",
        "wins": 11,
        "losses": 23,
        "total_earnings": 610
      },
      {
        "player_id": 23,
        "username": "fake_sarah",
        "money": 4998,
        "hands_played": 301,
        "win_percentage": "66.45%",
        "wins": 200,
        "losses": 101,
        "total_earnings": 700
      },
      {
        "player_id": 24,
        "username": "fake_betty",
        "money": 4000,
        "hands_played": 40,
        "win_percentage": "62.50%",
        "wins": 25,
        "losses": 15,
        "total_earnings": 800
      },
      {
        "player_id": 25,
        "username": "fake_betsy",
        "money": 3000,
        "hands_played": 400,
        "win_percentage": "25.00%",
        "wins": 100,
        "losses": 300,
        "total_earnings": 900
      },
      {
        "player_id": 26,
        "username": "fake_vinny",
        "money": 1030,
        "hands_played": 54,
        "win_percentage": "88.89%",
        "wins": 48,
        "losses": 6,
        "total_earnings": 0
      },
      {
        "player_id": 27,
        "username": "player1",
        "money": 1979,
        "hands_played": 4,
        "win_percentage": "75.00%",
        "wins": 3,
        "losses": 1,
        "total_earnings": 1954
      },
      {
        "player_id": 28,
        "username": "a",
        "money": 1000,
        "hands_played": 0,
        "win_percentage": "0.00%",
        "wins": 0,
        "losses": 0,
        "total_earnings": 0
      }
    ],
    gamesData: [],
    singleGameData: [],
  },
};

export const GamesData = {
  args: {
    playerData: [],
    gamesData: [
      {
        "game_id": 1,
        "game_variant": "5 Card Draw",
        "num_players": 4,
        "winners": ["player1"],
        "dealer_idx": 1
      },
      {
        "game_id": 2,
        "game_variant": "7 Card Stud",
        "num_players": 3,
        "winners": ["player1"],
        "dealer_idx": 1
      },
      {
        "game_id": 3,
        "game_variant": "Texas Hold Em",
        "num_players": 3,
        "winners": ["fake_wanda"],
        "dealer_idx": 1
      },
      {
        "game_id": 4,
        "game_variant": "Texas Hold Em",
        "num_players": 3,
        "winners": ["player1"],
        "dealer_idx": 2
      },
      {
        "game_id": 5,
        "game_variant": "5 Card Draw",
        "num_players": 1,
        "winners": ["1"],
        "dealer_idx": 0
      }
    ],
    singleGameData: [],
  },
};

export const SingleGameData = {
  args: {
    playerData: [],
    gamesData: [
      {
        "game_id": 1,
        "game_variant": "5 Card Draw",
        "num_players": 4,
        "winners": ["player1"],
        "dealer_idx": 1
      },
      {
        "game_id": 2,
        "game_variant": "7 Card Stud",
        "num_players": 3,
        "winners": ["player1"],
        "dealer_idx": 1
      },
      {
        "game_id": 3,
        "game_variant": "Texas Hold Em",
        "num_players": 3,
        "winners": ["fake_wanda"],
        "dealer_idx": 1
      },
      {
        "game_id": 4,
        "game_variant": "Texas Hold Em",
        "num_players": 3,
        "winners": ["player1"],
        "dealer_idx": 2
      },
      {
        "game_id": 5,
        "game_variant": "5 Card Draw",
        "num_players": 1,
        "winners": ["1"],
        "dealer_idx": 0
      }
    ],
    singleGameData: [
      {
        "player_id": 27,
        "player_name": "player1",
        "hand": ["8S", "8D", "3S", "AD", "KD"],
        "total_wagered": 550,
        "chips_won": 1107
      },
      {
        "player_id": 26,
        "player_name": "fake_vinny",
        "hand": ["3D", "QH", "8H", "8C", "4S"],
        "total_wagered": 550,
        "chips_won": 0
      },
      {
        "player_id": 23,
        "player_name": "fake_sarah",
        "hand": ["4H", "3C", "7H", "9S", "TH"],
        "total_wagered": 2,
        "chips_won": 0
      },
      {
        "player_id": 22,
        "player_name": "fake_wanda",
        "hand": ["TS", "AC", "6S", "JH", "6C"],
        "total_wagered": 5,
        "chips_won": 0
      }
    ],
  },
};
