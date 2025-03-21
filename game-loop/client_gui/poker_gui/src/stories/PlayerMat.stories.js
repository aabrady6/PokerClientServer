import { action } from '@storybook/addon-actions';

import PlayerMat from '@/components/PlayerMat.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/PlayerMat',
  component: PlayerMat,
  tags: ['autodocs'],
  render: (args) => ({
    components: { PlayerMat },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<PlayerMat v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    },
    type: "full",
  },
};

export const FaceDownDown = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
      lastMove: "Check",
    },
    type: "full"
  },
};

export const FaceDownSomeCards = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false]],
      faceDownCards: [["", false], ["", false], ["JC", false]],
      lastMove: "Check",
    },
    type: "full",
  },
};

export const FaceDownSomeCardsOverlapped = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false]],
      faceDownCards: ["", "", ""],
      lastMove: "Check",
    },
    type: "overlap",
  },
};

export const Dealer = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    },
    type: "full",
    token: "D",
  },
};

export const BigBlind = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    },
    type: "full",
    token: "BB",
  },
};

export const SmallBlind = {
  args: {
    player: {
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    },
    type: "full",
    token: "SB",
  },
};