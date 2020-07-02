import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import * as AppButton from './AppButton';

export const StyledBlueButton = styled(AppButton.BlueButton)({
  marginBottom: 18,
});

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
    paddingTop: 18,
    paddingBottom: 22,
    paddingHorizontal: 22,
    flexDirection: 'column',
    flex: 0,
  }),
  support__status_icon: Styles.createViewStyle({
    alignItems: 'center',
    marginBottom: 32,
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
    // TODO: Use bigText in comonStyles when converted from ReactXP
    fontFamily: 'DINPro',
    fontSize: 30,
    fontWeight: '900',
    lineHeight: 34,
    color: colors.white,
    marginBottom: 4,
  }),
};
