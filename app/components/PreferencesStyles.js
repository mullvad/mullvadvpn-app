// @flow
import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    preferences: {
      backgroundColor: colors.darkBlue,
      flex: 1,
    },
    preferences__container: {
      display: 'flex',
      flexDirection: 'column',
      flex: 1,
    },
    preferences__header: {
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      paddingTop: 16,
      paddingRight: 24,
      paddingLeft: 24,
      paddingBottom: 24,
    },
    preferences__content: {
      flexDirection: 'column',
      flexGrow: 1,
      flexShrink: 1,
      flexBasis: 'auto',
    },
    preferences__cell: {
      backgroundColor: colors.blue,
      flexDirection: 'row',
      alignItems: 'center',
    },
    preferences__cell_accessory: {
      marginRight: 12,
    },
    preferences__cell_footer: {
      paddingTop: 8,
      paddingRight: 24,
      paddingBottom: 24,
      paddingLeft: 24,
    },
    preferences__cell_label_container: {
      paddingTop: 14,
      paddingRight: 12,
      paddingBottom: 14,
      paddingLeft: 24,
      flexGrow: 1,
    },
  }),
  ...createTextStyles({
    preferences__title: {
      fontFamily: 'DINPro',
      fontSize: 32,
      fontWeight: '900',
      lineHeight: 40,
      color: colors.white,
    },
    preferences__cell_label: {
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      letterSpacing: -0.2,
      color: colors.white,
    },
    preferences__cell_footer_label: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: colors.white80,
    },
  }),
};
