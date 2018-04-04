// @flow
import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    select_location: {
      backgroundColor: colors.darkBlue,
      height: '100%',
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
      marginLeft: 8,
      marginRight: 8,
    },
    relay_status__inactive: {
      backgroundColor: colors.red95,
    },
    relay_status__active: {
      backgroundColor: colors.green90,
    },
    tick_icon: {
      marginLeft: 4,
      marginRight: 4,
    },
    country: {
      flexDirection: 'column',
      flex: 0,
    },
    collapse_button: {
      height: 24,
      width: 24,
      alignSelf: 'flex-end',
    },
    sub_cell: {
      paddingLeft: 40,
      backgroundColor: colors.blue40,
    },
    sub_cell__selected: {
      paddingLeft: 40,
      backgroundColor: colors.green,
    },
    cell_selected: {
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