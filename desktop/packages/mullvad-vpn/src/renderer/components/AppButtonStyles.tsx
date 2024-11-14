import styled from 'styled-components';

import { buttonText } from './common-styles';

export const StyledLabel = styled.span<{ $textOffset: number }>(buttonText, (props) => ({
  paddingLeft: props.$textOffset > 0 ? `${props.$textOffset}px` : 0,
  paddingRight: props.$textOffset < 0 ? `${-props.$textOffset}px` : 0,
  textAlign: 'center',
  wordBreak: 'break-word',
}));

export const StyledButtonContent = styled.div({
  flex: 1,
  display: 'grid',
  gridTemplateColumns: '1fr auto 1fr',
  alignItems: 'center',
  padding: '9px',
});

export const transparentButton = {
  backdropFilter: 'blur(4px)',
};

export const StyledLeft = styled.div({
  justifySelf: 'start',
  display: 'flex',
  flexDirection: 'column',
});

export const StyledRight = styled(StyledLeft)({
  justifySelf: 'end',
});

export const StyledVisibleSide = styled.div({
  display: 'flex',
  flexDirection: 'row',
});

export const StyledHiddenSide = styled(StyledVisibleSide).attrs({ 'aria-hidden': true })({
  height: 0,
  visibility: 'hidden',
});
