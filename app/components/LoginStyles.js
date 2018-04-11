// @flow
import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    login_footer: {
      backgroundColor: colors.darkBlue,
      paddingTop: 18,
      paddingBottom:16,
      flex: 0,
    },
    status_icon: {
      flex: 0,
      height: 48,
      marginBottom: 30,
      justifyContent: 'center',
    },
    login_form:{
      flex:1,
      flexDirection: 'column',
      overflow:'visible',
      paddingTop: 0,
      paddingBottom: 0,
      paddingLeft: 24,
      paddingRight: 24,
      marginTop: 83,
      marginBottom:0,
      marginRight: 0,
      marginLeft:0,
    },
    account_input_group: {
      borderWidth: 2,
      borderRadius: 8,
      borderColor: 'transparent',
    },
    account_input_group__active: {
      borderColor: colors.darkBlue,
    },
    account_input_group__inactive: {
      opacity: 0.6,
    },
    account_input_group__error: {
      borderColor: colors.red40,
      color: colors.red,
    },
    account_input_textfield: {
      color: colors.blue,
    },
    account_input_backdrop: {
      backgroundColor: colors.white,
      borderColor: colors.darkBlue,
      flexDirection: 'row',
    },
    account_input_textfield__inactive: {
      backgroundColor: colors.white60,
    },
    input_button: {
      flex: 0,
      borderWidth: 0,
      width: 48,
      alignItems: 'center',
      justifyContent: 'center',
    },
    input_button__invisible: {
      backgroundColor: colors.white,
      opacity: 0,
    },
    input_arrow: {
      flex: 0,
      borderWidth: 0,
      width: 48,
      alignItems: 'center',
      justifyContent: 'center',
      color: colors.blue20,
    },
    input_arrow__active: {
      color: colors.white,
    },
    input_arrow__invisible: {
      color: colors.white,
      opacity: 0,
    },

    account_dropdown__spacer: {
      height: 1,
      backgroundColor: colors.darkBlue,
    },
    account_dropdown__item: {
      paddingTop: 10,
      paddingRight: 12,
      paddingLeft: 12,
      paddingBottom: 12,
      marginBottom: 0,
      flexDirection: 'row',
      backgroundColor: colors.white60,
    },
    account_dropdown__item_hover: {
      backgroundColor: colors.white40,
    },
    account_dropdown__remove: {
      justifyContent: 'center',
      color: colors.blue40,
    },
    account_dropdown__remove_cell_hover: {
      color: colors.blue60,
    },
    account_dropdown__remove_hover: {
      color: colors.blue,
    },
    account_dropdown__label_hover: {
      color: colors.blue,
    },
  }),
  ...createTextStyles({
    login_footer__prompt: {
      color: colors.white80,
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 18,
      letterSpacing: -0.2,
      marginLeft: 24,
      marginRight: 24,
    },
    title: {
      fontFamily: 'DINPro',
      fontSize: 32,
      fontWeight: '900',
      lineHeight: 44,
      letterSpacing: -0.7,
      color: colors.white,
      marginBottom: 7,
      flex:0,
    },
    subtitle: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      letterSpacing: -0.2,
      color: colors.white80,
      marginBottom: 8,
    },
    account_input_textfield: {
      borderWidth: 0,
      paddingTop: 10,
      paddingRight: 12,
      paddingLeft: 12,
      paddingBottom: 12,
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: colors.blue,
      backgroundColor: 'transparent',
      flex: 1,
    },
    account_dropdown__label: {
      flex: 1,
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: colors.blue80,
      borderWidth: 0,
      textAlign: 'left',
      marginLeft: 0,
    },
  }),
};
