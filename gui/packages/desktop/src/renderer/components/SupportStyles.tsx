import { Styles } from 'reactxp';
import { colors } from '../../config.json';

export default {
  support: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  support__container: Styles.createViewStyle({
    flexDirection: 'column',
    flex: 1,
  }),
  support__content: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
    justifyContent: 'space-between',
  }),
  support__form: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
  }),
  support__form_row: Styles.createViewStyle({
    paddingLeft: 22,
    paddingRight: 22,
    marginBottom: 12,
  }),
  support__form_row_email: Styles.createViewStyle({
    paddingLeft: 22,
    paddingRight: 22,
    marginBottom: 12,
  }),
  support__form_row_message: Styles.createViewStyle({
    flex: 1,
    paddingLeft: 22,
    paddingRight: 22,
  }),
  support__form_message_scroll_wrap: Styles.createViewStyle({
    flex: 1,
    borderRadius: 4,
    overflow: 'hidden',
  }),
  support__footer: Styles.createViewStyle({
    paddingTop: 16,
    paddingBottom: 16,
    paddingLeft: 24,
    paddingRight: 24,
    flexDirection: 'column',
    flex: 0,
  }),
  support__status_icon: Styles.createViewStyle({
    alignItems: 'center',
    marginBottom: 32,
  }),
  view_logs_button: Styles.createViewStyle({
    marginBottom: 16,
  }),
  edit_message_button: Styles.createViewStyle({
    marginBottom: 16,
  }),
  support__form_email: Styles.createTextStyle({
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
  }),
  support__form_message: Styles.createTextStyle({
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
  }),
  support__sent_message: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    overflow: 'visible',
    color: colors.white60,
    lineHeight: 20,
    letterSpacing: -0.2,
  }),
  support__sent_email: Styles.createTextStyle({
    fontWeight: '900',
    color: colors.white,
  }),
  support__status_security__secure: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '800',
    lineHeight: 22,
    marginBottom: 4,
    color: colors.green,
  }),
  support__send_status: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 34,
    fontWeight: '900',
    letterSpacing: -0.9,
    color: colors.white,
    marginBottom: 4,
  }),
  confirm_no_email_background: Styles.createViewStyle({
    flex: 1,
    justifyContent: 'center',
    paddingLeft: 14,
    paddingRight: 14,
  }),
  confirm_no_email_dialog: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    borderRadius: 11,
    padding: 16,
  }),
  confirm_no_email_warning: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '500',
    lineHeight: 20,
    color: colors.white80,
    marginBottom: 12,
  }),
  confirm_no_email_back_button: Styles.createViewStyle({
    marginTop: 16,
  }),
};
