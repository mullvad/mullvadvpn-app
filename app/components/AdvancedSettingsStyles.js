// @flow

import { createViewStyles, createTextStyles } from '../lib/styles';

export default {
  ...createViewStyles({
    advanced_settings: {
      backgroundColor: '#192E45',
      flex: 1,
    },
    advanced_settings__container: {
      flexDirection: 'column',
      flex: 1,
    },
    advanced_settings__header: {
      flexGrow: 0,
      flexShrink: 0,
      flexBasis: 'auto',
      paddingTop: 16,
      paddingRight: 24,
      paddingLeft: 24,
      paddingBottom: 24,
    },
    advanced_settings__close: {
      cursor: 'default',
    },
    advanced_settings__close_content: {
      flexDirection: 'row',
      alignItems: 'center',
      paddingLeft: 12,
      paddingTop: 24,
    },
    advanced_settings__close_icon: {
      opacity: 0.6,
      marginRight: 8,
    },
    advanced_settings__scrollview: {
      flexGrow: 1,
      flexShrink: 1,
      flexBasis: '100%',
    },
    advanced_settings__content: {
      flexDirection: 'column',
      flexGrow: 1,
      flexShrink: 0,
      flexBasis: 'auto',
    },
    advanced_settings__cell: {
      backgroundColor: '#44AD4D',
      flexDirection: 'row',
      paddingTop: 15,
      paddingBottom: 15,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      justifyContent: 'flex-start',
    },
    advanced_settings__cell_hover: {
      backgroundColor: 'rgba(41, 71, 115, 0.9)',
    },
    advanced_settings__cell_selected_hover: {
      backgroundColor: '#44AD4D',
    },
    advanced_settings__cell_spacer: {
      height: 24,
    },
    advanced_settings__cell_icon: {
      width: 24,
      height: 24,
      marginRight: 8,
      flex: 0,
      color: 'rgba(255, 255, 255, 0.8)',
    },
    advanced_settings__cell_dimmed: {
      paddingTop: 15,
      paddingBottom: 15,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      backgroundColor: 'rgb(36, 57, 84)',
      flexDirection: 'row',
      justifyContent: 'flex-start',
    },
    advanced_settings__cell_footer: {
      paddingTop: 8,
      paddingRight: 24,
      paddingBottom: 24,
      paddingLeft: 24,
    },
    advanced_settings__cell_label_container: {
      paddingTop: 15,
      paddingRight: 12,
      paddingBottom: 15,
      paddingLeft: 24,
      flexGrow: 1,
    },
  }),
  ...createTextStyles({
    advanced_settings__section_title: {
      backgroundColor: 'rgb(41, 71, 115)',
      paddingTop: 15,
      paddingBottom: 15,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: '#fff',
    },
    advanced_settings__close_title: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: 'rgba(255, 255, 255, 0.6)',
    },
    advanced_settings__title: {
      fontFamily: 'DINPro',
      fontSize: 32,
      fontWeight: '900',
      lineHeight: 40,
      color: '#fff',
    },
    advanced_settings__cell_label: {
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      letterSpacing: -0.2,
      color: '#fff',
      flex: 0,
    },
    advanced_settings__cell_footer_label: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: 'rgba(255,255,255,0.8)'
    }
  })
};