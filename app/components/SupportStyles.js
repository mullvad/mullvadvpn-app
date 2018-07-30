import { createViewStyles, createTextStyles } from '../lib/styles';
import { colors } from '../config';

export default Object.assign(
  createViewStyles({
    support: {
      backgroundColor: colors.darkBlue,
      flex: 1,
    },
    support__container: {
      display: 'flex',
      flexDirection: 'column',
      flex: 1,
    },
    support__content: {
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      justifyContent: 'space-between',
    },
    support__form: {
      display: 'flex',
      flex: 1,
      flexDirection: 'column',
    },
    support__form_row: {
      paddingLeft: 22,
      paddingRight: 22,
      marginBottom: 12,
    },
    support__form_row_email: {
      paddingLeft: 22,
      paddingRight: 22,
      marginBottom: 12,
    },
    support__form_row_message: {
      flex: 1,
      paddingLeft: 22,
      paddingRight: 22,
    },
    support__form_message_scroll_wrap: {
      flex: 1,
      display: 'flex',
      borderRadius: 4,
      overflow: 'hidden',
    },
    support__footer: {
      paddingTop: 16,
      paddingBottom: 16,
      paddingLeft: 24,
      paddingRight: 24,
      flexDirection: 'column',
      flex: 0,
    },
    support__status_icon: {
      textAlign: 'center',
      marginBottom: 32,
    },
    view_logs_button: {
      marginBottom: 16,
    },
    edit_message_button: {
      marginBottom: 16,
    },
  }),
  createTextStyles({
    support__form_email: {
      flex: 1,
      borderRadius: 4,
      overflow: 'hidden',
      paddingTop: 14,
      paddingLeft: 14,
      paddingRight: 14,
      paddingBottom: 14,
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 26,
      color: colors.blue,
      backgroundColor: colors.white,
    },
    support__form_message: {
      paddingTop: 14,
      paddingLeft: 14,
      paddingRight: 14,
      paddingBottom: 14,
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      color: colors.blue,
      backgroundColor: colors.white,
      flex: 1,
    },
    support__sent_message: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      overflow: 'visible',
      color: colors.white60,
      lineHeight: 20,
      letterSpacing: -0.2,
    },
    support__sent_email: {
      fontWeight: '900',
      color: colors.white,
    },
    support__status_security__secure: {
      fontFamily: 'Open Sans',
      fontSize: 16,
      fontWeight: '800',
      lineHeight: 22,
      marginBottom: 4,
      color: colors.green,
    },
    support__send_status: {
      fontFamily: 'DINPro',
      fontSize: 38,
      fontWeight: '900',
      maxHeight: 'calc(1.16em * 2)',
      overflow: 'visible',
      letterSpacing: -0.9,
      color: colors.white,
      marginBottom: 4,
    },
    support__no_email_warning: {
      fontFamily: 'Open Sans',
      fontSize: 13,
      lineHeight: 16,
      color: colors.white80,
      marginBottom: 12,
    },
  }),
);
