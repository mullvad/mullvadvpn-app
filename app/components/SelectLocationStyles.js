// @flow

import { createViewStyles } from '../lib/styles';
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
    title_header: {
      paddingBottom: 0,
    },
    subtitle_header: {
      paddingTop: 0,
    },
    content: {
      overflow: 'visible',
    },
    relay_status: {
      width: 16,
      height: 16,
      borderRadius: 8,
      marginLeft: 4,
      marginRight: 4,
      marginTop: 20,
      marginBottom: 20,
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
};
