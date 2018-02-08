// @flow

import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    account: {
      backgroundColor: colors.darkBlue,
      height: '100%',
    },
    account__container: {
      flexDirection: 'column',
      height: '100%',
      paddingBottom: 48,
    },
    account__header: {
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      paddingTop: 16,
      paddingRight: 24,
      paddingLeft: 24,
      paddingBottom: 12,
    },
    account__close: {
      flexDirection: 'row',
      alignItems: 'center',
      alignSelf: 'flex-start',
      marginLeft: 12,
      marginTop: 24,
      cursor: 'default',
    },
    account__close_icon: {
      width: 24,
      height: 24,
      opacity: 0.6,
      marginRight: 8,
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
      marginTop: 24,
    },
    account__footer: {
      marginTop: 12,
    },
    account__buymore: {
      marginTop: 12,
      marginLeft: 24,
      marginRight: 24,
      backgroundColor: colors.green,
    },
    account__buymore_hover: {
      backgroundColor: colors.green90,
    },
    account__buymore_icon: {
      width: 16,
      height: 16,
      marginRight: 8,
    },
    account__logout:{
      marginTop: 12,
      marginLeft: 24,
      marginRight: 24,
      marginBottom: 12,
      backgroundColor: colors.red,
    },
    account__logout_hover:{
      backgroundColor: colors.red95,
    },
  }),
  ...createTextStyles({
    account__close_title: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: colors.white60,
    },
    account__title: {
      fontFamily: 'DINPro',
      fontSize: 32,
      fontWeight: '900',
      lineHeight: 40,
      color: colors.white,
    },
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
    }
  })
};