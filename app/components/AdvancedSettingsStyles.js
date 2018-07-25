// @flow

import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default {
  ...createViewStyles({
    advanced_settings: {
      backgroundColor: colors.darkBlue,
      flex: 1,
    },
    advanced_settings__container: {
      flexDirection: 'column',
      flex: 1,
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
      overflow: 'visible',
    },
    advanced_settings__cell: {
      cursor: 'default',
      backgroundColor: colors.green,
      flexDirection: 'row',
      paddingTop: 14,
      paddingBottom: 14,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      justifyContent: 'flex-start',
    },
    advanced_settings__cell_hover: {
      backgroundColor: colors.blue80,
    },
    advanced_settings__cell_selected_hover: {
      backgroundColor: colors.green,
    },
    advanced_settings__cell_spacer: {
      height: 24,
    },
    advanced_settings__cell_icon: {
      width: 24,
      height: 24,
      marginRight: 8,
      flex: 0,
      color: colors.white80,
    },
    advanced_settings__cell_dimmed: {
      cursor: 'default',
      paddingTop: 14,
      paddingBottom: 14,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      backgroundColor: colors.blue40,
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
      paddingTop: 14,
      paddingRight: 12,
      paddingBottom: 14,
      paddingLeft: 24,
      flexGrow: 1,
    },
  }),
  ...createTextStyles({
    advanced_settings__section_title: {
      backgroundColor: colors.blue,
      paddingTop: 14,
      paddingBottom: 14,
      paddingLeft: 24,
      paddingRight: 24,
      marginBottom: 1,
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      color: colors.white,
    },
    advanced_settings__cell_label: {
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      letterSpacing: -0.2,
      color: colors.white,
      flex: 0,
    },
    advanced_settings__cell_footer_label: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: colors.white80,
    },
  }),
};
