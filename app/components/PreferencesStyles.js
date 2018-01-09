// @flow

import { createViewStyles, createTextStyles } from '../lib/styles';

export default {
  ...createViewStyles({
    preferences: {
      backgroundColor: '#192E45',
      height: '100%',
    },
    preferences__container: {
      display: 'flex',
      flexDirection: 'column',
      height: '100%',
    },
    preferences__header: {
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      paddingTop: 40,
      paddingRight: 24,
      paddingLeft: 24,
      paddingBottom: 24,
    },
    preferences__close: {
      position: 'absolute',
      top: 0,
      left: 12,
      borderWidth: 0,
      padding: 0,
      margin: 0,
      zIndex: 1, /* part of .preferences__container covers the button */
      cursor: 'default',
    },
    preferences__close_content: {
      flexDirection: 'row',
      alignItems: 'center',
    },
    preferences__close_icon: {
      opacity: 0.6,
      marginRight: 8,
    },
    preferences__content: {
      flexDirection: 'column',
      flexGrow: 1,
      flexShrink: 1,
      flexBasis: 'auto',
    },
    preferences__cell: {
      backgroundColor: 'rgba(41,71,115,1)',
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
      paddingTop: 15,
      paddingRight: 12,
      paddingBottom: 15,
      paddingLeft: 24,
      flexGrow: 1,
    },
  }),
  ...createTextStyles({
    preferences__close_title: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: 'rgba(255, 255, 255, 0.6)',
    },
    preferences__title: {
      fontFamily: 'DINPro',
      fontSize: 32,
      fontWeight: '900',
      lineHeight: 40,
      color: '#fff',
    },
    preferences__cell_label: {
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      letterSpacing: -0.2,
      color: '#fff',
    },
    preferences__cell_footer_label: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: 'rgba(255,255,255,0.8)'
    }
  })
};