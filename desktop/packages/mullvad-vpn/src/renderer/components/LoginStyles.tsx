import styled from 'styled-components';

import { Icon } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';
import { buttonReset } from '../lib/styles';
import * as Cell from './cell';
import { hugeText, largeText, measurements, smallText, tinyText } from './common-styles';
import FormattableTextInput from './FormattableTextInput';
import { Footer } from './Layout';

export const StyledAccountDropdownContainer = styled.ul({
  display: 'flex',
  flexDirection: 'column',
});

export const StyledInputSubmitIcon = styled(Icon)<{ $visible: boolean }>((props) => ({
  opacity: props.$visible ? 1 : 0,
}));

export const StyledAccountDropdownItem = styled.li({
  display: 'flex',
  flex: 1,
});

const baseButtonStyles = {
  ...buttonReset,
  width: '100%',
  height: '100%',
  backgroundColor: Colors.white60,
  cursor: 'default',
  '&&:hover': {
    backgroundColor: Colors.white40,
  },
  '&:focus-visible': {
    outline: `2px solid ${Colors.white}`,
    outlineOffset: '-2px',
  },
};

export const StyledAccountDropdownItemButton = styled.button({
  ...baseButtonStyles,
  paddingLeft: Spacings.medium,
});

export const StyledAccountDropdownItemIconButton = styled.button({
  ...baseButtonStyles,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
});

export const StyledAccountDropdownTrailingButton = styled.button({
  ...buttonReset,
  backgroundColor: 'transparent',
  cursor: 'pointer',
  '&:focus-visible': {
    outline: `2px solid ${Colors.white}`,
    outlineOffset: '2px',
  },
});

export const StyledAccountDropdownItemButtonLabel = styled(Cell.Label)(largeText, {
  margin: '0',
  color: Colors.blue80,
  borderWidth: 0,
  textAlign: 'left',
  marginLeft: 0,
  cursor: 'default',
});

export const StyledTopInfo = styled.div({
  display: 'flex',
  justifyContent: 'center',
  flex: 1,
});

export const StyledFooter = styled(Footer)<{ $show: boolean }>((props) => ({
  position: 'relative',
  width: '100%',
  bottom: 0,
  transform: `translateY(${props.$show ? 0 : 100}%)`,
  backgroundColor: Colors.darkBlue,
  transition: 'transform 250ms ease-in-out',
}));

export const StyledStatusIcon = styled.div({
  display: 'flex',
  alignSelf: 'end',
  flex: 0,
  marginBottom: '30px',
  justifyContent: 'center',
  height: '48px',
  minHeight: '48px',
});

export const StyledLoginForm = styled.div({
  display: 'flex',
  flex: '0 1 225px',
  flexDirection: 'column',
  overflow: 'visible',
  padding: `0 ${measurements.horizontalViewMargin}`,
});

interface IStyledAccountInputGroupProps {
  $editable: boolean;
  $active: boolean;
  $error: boolean;
}

export const StyledAccountInputGroup = styled.form<IStyledAccountInputGroupProps>((props) => ({
  borderWidth: '2px',
  borderStyle: 'solid',
  borderRadius: '8px',
  overflow: 'hidden',
  borderColor: props.$error ? Colors.red40 : props.$active ? Colors.darkBlue : 'transparent',
  opacity: props.$editable ? 1 : 0.6,
}));

export const StyledAccountInputBackdrop = styled.div({
  display: 'flex',
  backgroundColor: Colors.white,
  borderColor: Colors.darkBlue,
});

export const StyledInputButton = styled.button<{ $visible: boolean }>((props) => ({
  display: 'flex',
  borderWidth: 0,
  width: '48px',
  alignItems: 'center',
  justifyContent: 'center',
  opacity: props.$visible ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
  backgroundColor: Colors.green,
}));

export const StyledDropdownSpacer = styled.div({
  height: 1,
  backgroundColor: Colors.darkBlue,
});

export const StyledTitle = styled.h1(hugeText, {
  lineHeight: '40px',
  marginBottom: '7px',
  flex: 0,
});

export const StyledInput = styled(FormattableTextInput)(largeText, {
  fontWeight: 700,
  minWidth: 0,
  borderWidth: 0,
  padding: '12px 12px 12px',
  color: Colors.blue,
  backgroundColor: 'transparent',
  flex: 1,
  '&&::placeholder': {
    color: Colors.blue40,
  },
});

export const StyledBlockMessageContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  alignSelf: 'start',
  backgroundColor: Colors.darkBlue,
  borderRadius: '8px',
  margin: '5px 16px 10px',
  padding: '16px',
});

export const StyledBlockTitle = styled.div(smallText, {
  color: Colors.white,
  marginBottom: '5px',
  fontWeight: 700,
});

export const StyledBlockMessage = styled.div(tinyText, {
  color: Colors.white,
  marginBottom: '10px',
});
