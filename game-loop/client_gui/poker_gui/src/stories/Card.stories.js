import { action } from '@storybook/addon-actions';

import Card from '@/components/Card.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/Card',
  component: Card,
  tags: ['autodocs'],
  render: (args) => ({
    components: { Card },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<Card v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
  },
};

export const FiveH = {
  args: {
    rank: "5",
    suit: "H"
  },
};

export const JackD = {
  args: {
    rank: "J",
    suit: "D"
  },
};

export const AceC = {
  args: {
    rank: "A",
    suit: "C"
  },
};

export const TenS = {
  args: {
    rank: "T",
    suit: "S"
  },
};

export const Selected = {
  args: {
    rank: "T",
    suit: "S",
    selected: true,
  },
};
