import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import ImageView from './ImageView';
import * as Cell from './Cell';

export const AccountDropdownRemoveIcon = styled(ImageView)({
  justifyContent: 'center',
  paddingTop: '10px',
  paddingRight: '12px',
  paddingBottom: '12px',
  paddingLeft: '12px',
  marginLeft: '0px',
});

export const InputSubmitIcon = styled(ImageView)((props: { visible: boolean }) => ({
  flex: 0,
  borderWidth: '0px',
  width: '48px',
  alignItems: 'center',
  justifyContent: 'center',
  opacity: props.visible ? 1 : 0,
}));

export const AccountDropdownItemButton = styled(Cell.CellButton)({
  padding: '0px',
  marginBottom: '0px',
  flexDirection: 'row',
  alignItems: 'stretch',
  backgroundColor: colors.white60,
  cursor: 'default',
  ':not(:disabled):hover': {
    backgroundColor: colors.white40,
  },
});

export const AccountDropdownItemButtonLabel = styled(Cell.Label)({
  padding: '11px 0px 11px 12px',
  margin: '0',
  color: colors.blue80,
  borderWidth: 0,
  textAlign: 'left',
  marginLeft: 0,
  cursor: 'default',
  [AccountDropdownItemButton + ':hover']: {
    color: colors.blue,
  },
});

export default {
  login_footer: Styles.createViewStyle({
    flex: 0,
    paddingTop: 18,
    paddingBottom: 22,
    paddingHorizontal: 22,
    backgroundColor: colors.darkBlue,
  }),
  status_icon: Styles.createViewStyle({
    flex: 0,
    marginBottom: 30,
    alignItems: 'center',
    height: 48,
  }),
  login_form: Styles.createViewStyle({
    flex: 1,
    flexDirection: 'column',
    overflow: 'visible',
    paddingTop: 0,
    paddingBottom: 0,
    paddingLeft: 22,
    paddingRight: 22,
    marginTop: 83,
    marginBottom: 0,
    marginRight: 0,
    marginLeft: 0,
  }),
  account_input_group: Styles.createViewStyle({
    borderWidth: 2,
    borderRadius: 8,
    borderColor: 'transparent',
  }),
  account_input_group__active: Styles.createViewStyle({
    borderColor: colors.darkBlue,
  }),
  account_input_group__inactive: Styles.createViewStyle({
    opacity: 0.6,
  }),
  account_input_group__error: Styles.createViewStyle({
    borderColor: colors.red40,
  }),
  account_input_backdrop: Styles.createViewStyle({
    backgroundColor: colors.white,
    borderColor: colors.darkBlue,
    flexDirection: 'row',
  }),
  input_button: Styles.createViewStyle({
    flex: 0,
    borderWidth: 0,
    width: 48,
    alignItems: 'center',
    justifyContent: 'center',
  }),
  input_button__invisible: Styles.createViewStyle({
    backgroundColor: colors.white,
    opacity: 0,
  }),
  account_dropdown__spacer: Styles.createViewStyle({
    height: 1,
    backgroundColor: colors.darkBlue,
  }),

  login_footer__prompt: Styles.createTextStyle({
    color: colors.white80,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '600',
    lineHeight: 18,
    marginBottom: 8,
  }),
  // TODO: Use bigText in comonStyles when converted from ReactXP
  title: Styles.createTextStyle({
    fontFamily: 'DINPro',
    fontSize: 30,
    fontWeight: '900',
    lineHeight: 40,
    color: colors.white,
    marginBottom: 7,
    flex: 0,
  }),
  subtitle: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 13,
    lineHeight: 15,
    fontWeight: '600',
    color: colors.white80,
    marginBottom: 8,
  }),
  account_input_textfield: Styles.createTextInputStyle({
    borderWidth: 0,
    paddingTop: 10,
    paddingRight: 12,
    paddingLeft: 12,
    paddingBottom: 12,
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    color: colors.blue,
    backgroundColor: 'transparent',
    flex: 1,
  }),
};
