import { action } from '@storybook/addon-actions';

import ChooseGameType from '@/components/ChooseGameType.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/ChooseGameType',
  component: ChooseGameType,
  tags: ['autodocs'],
  render: (args) => ({
    components: { ChooseGameType },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<ChooseGameType v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    dealerOptions: ["Five-Card Draw", "Seven-Card Stud", "Texas Hold 'Em"],
  },
};
