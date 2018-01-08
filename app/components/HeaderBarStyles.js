// @flow
import { createTextStyles, createViewStyles } from '../lib/styles';

export default {
  ...createViewStyles({
    headerbar: {
      paddingTop: 12,
      paddingBottom: 12,
      paddingLeft: 12,
      paddingRight: 12,
      backgroundColor: '#294D73',
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
      backgroundColor: '#192E45',
    },
    headerbar__style_error: {
      backgroundColor: '#D0021B',
    },
    headerbar__style_success: {
      backgroundColor: '#44AD4D',
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
      color: 'rgba(255,255,255,0.6)',
      marginLeft: 8,
    }
  })
};
