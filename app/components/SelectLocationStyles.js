// @flow
import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    select_location: {
      backgroundColor: colors.darkBlue,
      flex: 1,
    },
    container: {
      flexDirection: 'column',
      flex: 1,
    },
    header:{
      flex: 0,
      marginBottom: 16,
    },
    close: {
      marginLeft: 12,
      marginTop: 24,
      cursor: 'default',
    },
    close_icon:{
      width: 24,
      height: 24,
      flex: 0,
      opacity: 0.6,
    },
    relay_status: {
      width: 16,
      height: 16,
      borderRadius: 8,
      marginLeft: 4,
      marginRight: 4,
      marginTop: 19.5,
      marginBottom: 19.5,
    },
    relay_status__inactive: {
      backgroundColor: colors.red95,
    },
    relay_status__active: {
      backgroundColor: colors.green90,
    },
    tick_icon: {
      color: colors.white,
      marginLeft: 0,
      marginRight: 0,
      marginTop: 15.5,
      marginBottom: 15.5,
    },
    country: {
      flexDirection: 'column',
      flex: 0,
    },
    collapse_button: {
      flex: 0,
      alignSelf: 'stretch',
      justifyContent: 'center',
      paddingRight: 16,
      paddingLeft: 16,
    },
    cell: {
      paddingTop: 0,
      paddingBottom: 0,
      paddingLeft: 20,
      paddingRight: 0,
    },
    sub_cell: {
      paddingTop: 0,
      paddingBottom: 0,
      paddingRight: 0,
      paddingLeft: 40,
      backgroundColor: colors.blue40,
    },
    sub_cell__selected: {
      paddingTop: 0,
      paddingBottom: 0,
      paddingRight: 0,
      paddingLeft: 40,
      backgroundColor: colors.green,
    },
    cell_selected: {
      paddingTop: 0,
      paddingBottom: 0,
      paddingLeft: 20,
      paddingRight: 0,
      backgroundColor: colors.green,
    },
    expand_chevron_hover: {
      color: colors.white,
    },
  }),
  ...createTextStyles({
    title: {
      fontFamily: 'DINPro',
      fontSize: 32,
      fontWeight: '900',
      lineHeight: 40,
      color: colors.white,
      paddingTop: 16,
      paddingLeft: 24,
      paddingRight: 24,
    },
    subtitle: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      color: colors.white60,
      letterSpacing: -0.2,
      paddingTop: 0,
      paddingLeft: 24,
      paddingRight: 24,
      paddingBottom: 24,
      flex: 0,
    },
  }),
};