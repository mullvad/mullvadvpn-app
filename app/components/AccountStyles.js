// @flow

import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    account: {
      backgroundColor: colors.darkBlue,
      flex: 1,
    },
    account__container: {
      flexDirection: 'column',
      flex: 1,
      paddingBottom: 48,
    },
    account__scrollview: {
      flexGrow: 1,
      flexShrink: 1,
      flexBasis: '100%',
    },
    account__content: {
      flexDirection: 'column',
      flexGrow: 1,
      flexShrink: 0,
      flexBasis: 'auto',
    },
    account__main: {
      marginBottom: 24,
    },
    account__row: {
      paddingTop: 0,
      paddingBottom: 0,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 24,
    },
    account__footer: {
      paddingLeft: 24,
      paddingRight: 24,
    },
    account__buy_button: {
      marginBottom: 24,
    },
  }),
  ...createTextStyles({
    account__row_label: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: colors.white60,
      marginBottom: 9,
    },
    account__row_value: {
      fontFamily: 'Open Sans',
      fontSize: 16,
      lineHeight: 19,
      fontWeight: '800',
      color: colors.white,
    },
    account__out_of_time: {
      fontFamily: 'Open Sans',
      fontSize: 16,
      fontWeight: '800',
      color: colors.red,
    },
    account__footer_label: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: colors.white80,
    },
  }),
};
