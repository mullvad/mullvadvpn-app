import styled from 'styled-components';

import { Flex, Icon } from '../../../lib/components';
import { colors, spacings } from '../../../lib/foundations';
import { hugeText, largeText, smallText, tinyText } from '../../common-styles';
import FormattableTextInput from '../../FormattableTextInput';

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
  width: '100%',
  height: '100%',
  backgroundColor: colors.whiteAlpha60,
  cursor: 'default',
  '&&:hover': {
    backgroundColor: colors.whiteAlpha40,
  },
  '&:focus-visible': {
    outline: `2px solid ${colors.white}`,
    outlineOffset: '-2px',
  },
};

export const StyledAccountDropdownItemButton = styled.button({
  ...baseButtonStyles,
  paddingLeft: spacings.medium,
});

export const StyledAccountDropdownItemIconButton = styled.button({
  ...baseButtonStyles,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
});

export const StyledTopInfo = styled.div({
  display: 'flex',
  justifyContent: 'center',
  flex: 1,
});

export const StyledFooter = styled(Flex)<{ $show: boolean }>((props) => ({
  position: 'absolute',
  width: '100%',
  bottom: 0,
  transform: `translateY(${props.$show ? 0 : 100}%)`,
  backgroundColor: colors.darkBlue,
  transition: 'transform 250ms ease-in-out',
}));

export const StyledStatusIcon = styled.div({
  display: 'flex',
  alignSelf: 'end',
  flex: 0,
  justifyContent: 'center',
  height: '48px',
  minHeight: '48px',
});

export const StyledLoginForm = styled.div({
  display: 'flex',
  flex: '0 1 225px',
  flexDirection: 'column',
  overflow: 'visible',
});

interface IStyledAccountInputGroupProps {
  $editable: boolean;
  $active: boolean;
  $error: boolean;
}

export const StyledAccountInputGroup = styled.div<IStyledAccountInputGroupProps>((props) => ({
  borderWidth: '2px',
  borderStyle: 'solid',
  borderRadius: '8px',
  overflow: 'hidden',
  borderColor: props.$error ? colors.red40 : props.$active ? colors.darkBlue : colors.transparent,
  opacity: props.$editable ? 1 : 0.6,
}));

export const StyledAccountInputBackdrop = styled.div({
  display: 'flex',
  backgroundColor: colors.white,
  borderColor: colors.darkBlue,
});

export const StyledDropdownSpacer = styled.div({
  height: 1,
  backgroundColor: colors.darkBlue,
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
  color: colors.blue,
  backgroundColor: 'transparent',
  flex: 1,
  '&&::placeholder': {
    color: colors.whiteOnBlue60,
  },
});

export const StyledBlockMessageContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  alignSelf: 'start',
  backgroundColor: colors.darkBlue,
  borderRadius: '8px',
  padding: '16px',
});

export const StyledBlockTitle = styled.div(smallText, {
  color: colors.white,
  marginBottom: '5px',
  fontWeight: 700,
});

export const StyledBlockMessage = styled.div(tinyText, {
  color: colors.white,
  marginBottom: '10px',
});
