// @flow
import { createViewStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    layout: {
      flexDirection: 'column',
      flex: 1,
    },
    header: {
      flex: 0,
    },
    container: {
      flex: 1,
      backgroundColor: colors.blue,
      overflow: 'hidden',
    }
  }),
};
