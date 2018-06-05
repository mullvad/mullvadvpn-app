// @flow
import { createViewStyles } from '../lib/styles';

export default {
  ...createViewStyles({
    darwin: {
      paddingTop: 24,
    },
    linux: {
      WebkitAppRegion: 'drag',
    },
    settings_icon: {
      WebkitAppRegion: 'no-drag',
    },
  }),
};
