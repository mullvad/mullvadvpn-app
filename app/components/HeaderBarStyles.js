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
    headerbar__hidden: {
      paddingTop: 24,
      paddingBottom: 0,
      paddingLeft: 0,
      paddingRight: 0,
    },
    headerbar__darwin: {
      paddingTop: 24,
    },
    headerbar__style_defaultDark: {
      backgroundColor: colors.darkBlue,
    },
    headerbar__style_error: {
      backgroundColor: colors.red,
    },
    headerbar__style_success: {
      backgroundColor: colors.green,
    },
    headerbar__container: {
      display: 'flex',
      flexDirection: 'row',
      alignItems: 'center',
    },
    headerbar__logo: {
      height: 50,
      width: 50,
    },
    headerbar__settings: {
      padding: 0
    },
    headerbar__settings_icon: {
      width: 24,
      height: 24,
    },
  }),
  ...createTextStyles({
    headerbar__title: {
      fontFamily: 'DINPro',
      fontSize: 24,
      fontWeight: '900',
      lineHeight: 30,
      letterSpacing: -0.5,
      color: colors.white60,
      marginLeft: 8,
    }
  })
};
