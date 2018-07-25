import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    settings: {
      backgroundColor: colors.darkBlue,
      flex: 1,
    },
    settings__container: {
      flexDirection: 'column',
      flex: 1,
    },
    settings__content: {
      flexDirection: 'column',
      flex: 1,
      justifyContent: 'space-between',
      overflow: 'visible',
    },
    settings__scrollview: {
      flexGrow: 1,
      flexShrink: 1,
      flexBasis: '100%',
    },
    settings__cell_spacer: {
      height: 24,
      flex: 0,
    },
    settings__footer: {
      paddingTop: 24,
      paddingBottom: 24,
      paddingLeft: 24,
      paddingRight: 24,
    },
  }),
  ...createTextStyles({
    settings__account_paid_until_label__error: {
      color: colors.red,
    },
  }),
};
