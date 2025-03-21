import { fn } from '@storybook/test';

import OverlappedCards from '@/components/OverlappedCards.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/OverlappedCards',
  component: OverlappedCards,
  tags: ['autodocs'],
  // Use `fn` to spy on the onClick arg, which will appear in the actions panel once invoked: https://storybook.js.org/docs/essentials/actions#action-args
  // args: { onClick: fn() },
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    cards: [{
      rank: "J",
      suit: "D"
    }, {
      rank: "J",
      suit: "D"
    }, {
      rank: "J",
      suit: "D"
    }],
  }
};

export const FiveCards = {
  args: {
    cards: [{
      rank: "J",
      suit: "D"
    }, {
      rank: "J",
      suit: "D"
    }, {
      rank: "J",
      suit: "D"
    }, {
      rank: "J",
      suit: "D"
    }, {
      rank: "J",
      suit: "D"
    }],
  }
};

export const OneCard = {
  args: {
    cards: [{
      rank: "J",
      suit: "D"
    }],
  }
};
