import styled from 'styled-components';
import { colors } from '../../config.json';
import ImageView from './ImageView';
import * as Cell from './cell';
import { bigText, smallText, sourceSansPro } from './common-styles';
import FormattableTextInput from './FormattableTextInput';

export const StyledAccountDropdownContainer = styled.ul({
  display: 'flex',
  flexDirection: 'column',
});

export const StyledAccountDropdownRemoveButton = styled.button({
  border: 'none',
  background: 'none',
});

export const StyledAccountDropdownRemoveIcon = styled(ImageView)({
  justifyContent: 'center',
  paddingTop: '10px',
  paddingRight: '12px',
  paddingBottom: '12px',
  paddingLeft: '12px',
  marginLeft: '0px',
});

export const StyledInputSubmitIcon = styled(ImageView)((props: { visible: boolean }) => ({
  flex: 0,
  borderWidth: '0px',
  width: '48px',
  alignItems: 'center',
  justifyContent: 'center',
  opacity: props.visible ? 1 : 0,
}));

export const StyledAccountDropdownItem = styled.li({
  display: 'flex',
  flex: 1,
  backgroundColor: colors.white60,
  cursor: 'default',
  ':hover': {
    backgroundColor: colors.white40,
  },
});

export const StyledAccountDropdownItemButton = styled(Cell.CellButton)({
  padding: '0px',
  marginBottom: '0px',
  flexDirection: 'row',
  alignItems: 'stretch',
  backgroundColor: 'transparent',
  ':not(:disabled):hover': {
    backgroundColor: 'transparent',
  },
});

export const StyledAccountDropdownItemButtonLabel = styled(Cell.Label)({
  padding: '11px 0px 11px 12px',
  margin: '0',
  color: colors.blue80,
  borderWidth: 0,
  textAlign: 'left',
  marginLeft: 0,
  cursor: 'default',
  [StyledAccountDropdownItemButton + ':hover']: {
    color: colors.blue,
  },
});

export const StyledFooter = styled.div({}, (props: { show: boolean }) => ({
  position: 'absolute',
  width: '100%',
  bottom: 0,
  transform: `translateY(${props.show ? 0 : 100}%)`,
  display: 'flex',
  flexDirection: 'column',
  padding: '18px 22px 22px',
  backgroundColor: colors.darkBlue,
  transition: 'transform 250ms ease-in-out',
}));

export const StyledStatusIcon = styled.div({
  display: 'flex',
  flex: 0,
  marginBottom: '30px',
  justifyContent: 'center',
  height: '48px',
  minHeight: '48px',
});

export const StyledLoginForm = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  overflow: 'visible',
  padding: '0 22px',
  margin: '83px 0 0',
});

interface IStyledAccountInputGroupProps {
  editable: boolean;
  active: boolean;
  error: boolean;
}

export const StyledAccountInputGroup = styled.form((props: IStyledAccountInputGroupProps) => ({
  borderWidth: '2px',
  borderStyle: 'solid',
  borderRadius: '8px',
  overflow: 'hidden',
  borderColor: props.error ? colors.red40 : props.active ? colors.darkBlue : 'transparent',
  opacity: props.editable ? 1 : 0.6,
}));

export const StyledAccountInputBackdrop = styled.div({
  display: 'flex',
  backgroundColor: colors.white,
  borderColor: colors.darkBlue,
});

export const StyledInputButton = styled.button((props: { visible: boolean }) => ({
  display: 'flex',
  flex: 0,
  borderWidth: 0,
  width: '48px',
  alignItems: 'center',
  justifyContent: 'center',
  opacity: props.visible ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
  backgroundColor: colors.green,
}));

export const StyledDropdownSpacer = styled.div({
  height: 1,
  backgroundColor: colors.darkBlue,
});

export const StyledLoginFooterPrompt = styled.span({
  color: colors.white80,
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '18px',
  marginBottom: '8px',
});

export const StyledTitle = styled.h1(bigText, {
  lineHeight: '40px',
  marginBottom: '7px',
  flex: 0,
});

export const StyledSubtitle = styled.span(smallText, {
  lineHeight: '15px',
  marginBottom: '8px',
});

export const StyledInput = styled(FormattableTextInput)({
  ...sourceSansPro,
  minWidth: 0,
  borderWidth: 0,
  padding: '10px 12px 12px',
  fontSize: '20px',
  lineHeight: '26px',
  color: colors.blue,
  backgroundColor: 'transparent',
  flex: 1,
  '::placeholder': {
    color: colors.blue40,
  },
});
