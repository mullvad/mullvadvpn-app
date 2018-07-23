// @flow
import { createTextStyles, createViewStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    headerbar: {
      paddingTop: 12,
      paddingBottom: 12,
      paddingLeft: 12,
      paddingRight: 12,
      backgroundColor: colors.blue,
      flexDirection: 'row',
      justifyContent: 'space-between',
      alignItems: 'center',
    },
    style_defaultDark: {
      backgroundColor: colors.darkBlue,
    },
    style_error: {
      backgroundColor: colors.red,
    },
    style_success: {
      backgroundColor: colors.green,
    },
    container: {
      display: 'flex',
      flexDirection: 'row',
      alignItems: 'center',
    },
    settings: {
      cursor: 'default',
      padding: 0,
    },
    settings_icon: {
      color: colors.white60,
    },
    settings_icon_hover: {
      color: colors.white,
    },
  }),
  ...createTextStyles({
    title: {
      fontFamily: 'DINPro',
      fontSize: 24,
      fontWeight: '900',
      lineHeight: 30,
      letterSpacing: -0.5,
      color: colors.white60,
      marginLeft: 8,
    },
  }),
};
