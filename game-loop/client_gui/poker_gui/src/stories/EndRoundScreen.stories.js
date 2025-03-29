import { action } from '@storybook/addon-actions';

import EndRoundScreen from '@/components/EndRoundScreen.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/EndRoundScreen',
  component: EndRoundScreen,
  tags: ['autodocs'],
  render: (args) => ({
    components: { EndRoundScreen },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<EndRoundScreen v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    headerText: "Player 1 won!",
    tableOptions: ["Play again", "Leave table"],
    dealerOptions: [],
    isDealer: false,
  },
};

export const Dealer = {
  args: {
    headerText: "Player 1 won!",
    tableOptions: ["Play again", "Leave table"],
    dealerOptions: ["Five-Card Draw", "Seven-Card Stud", "Texas Hold 'Em"],
    isDealer: true,
  },
};
