import { action } from '@storybook/addon-actions';

import Controls from '@/components/Controls.vue';

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
export default {
  title: 'Example/Controls',
  component: Controls,
  tags: ['autodocs'],
  render: (args) => ({
    components: { Controls },
    setup() {
        const onClick = action('click');

        return { args, onClick };
    },
    template: `<Controls v-bind="args" @click="onClick" />`,
  }),
};

// More on writing stories with args: https://storybook.js.org/docs/writing-stories/args
export const Default = {
  args: {
    controls: [{
      name: "Fold",
      visible: true,
    }, {
      name: "Call",
      visible: true,
    }, {
      name: "Raise",
      visible: true,
      needsAmount: true,
      amountMin: 5,
      amountMax: 1000,
    }]
  },
};
