import { action } from '@storybook/addon-actions';

import Table from '@/components/Table.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/Table',
  component: Table,
  tags: ['autodocs'],
  render: (args) => ({
    components: { Table },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<Table v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    players: [{
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
      token: "D",
    }, {
      name: "Player2",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
      lastMove: "Check",
      token: "SB",
    }, {
      name: "Player3",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
      lastMove: "Check",
      token: "BB",
    }, {
      name: "Player4",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["", false], ["", false], ["", false], ["", false], ["", false]],
      lastMove: "Check",
    }],
    communityCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
    pot: 500,
    playerName: "Player1"
  },
};

export const TwoPlayer = {
  args: {
    players: [{
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
      token: "D",
    }, {
      name: "Player2",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
      token: "SB",
    }],
    communityCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
    pot: 500,
    playerName: "Player1"
  },
};

export const FullPlayers = {
  args: {
    players: [{
      name: "Player1",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    }, {
      name: "Player2",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    }, {
      name: "Player3",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
      token: "D",
    }, {
      name: "Player4",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
      token: "SB",
    }, {
      name: "Player5",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
      token: "BB",
    }, {
      name: "Player6",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    }, {
      name: "Player7",
      totalCash: 100,
      betCash: 10,
      faceUpCards: [["2H", false], ["3H", false], ["6C", false], ["JD", false]],
      faceDownCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
      lastMove: "Check",
    }],
    communityCards: [["AH", false], ["2D", false], ["TC", false], ["QS", false], ["7C", false]],
    pot: 500,
    playerName: "Player1"
  },
};