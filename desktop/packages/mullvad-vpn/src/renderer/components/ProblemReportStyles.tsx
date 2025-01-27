import styled from 'styled-components';

import { Colors } from '../lib/foundations';
import { hugeText, measurements, smallText } from './common-styles';

export const StyledContentContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const StyledContent = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  justifyContent: 'space-between',
});

export const StyledForm = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  margin: `0 ${measurements.horizontalViewMargin}`,
});

export const StyledFormEmailRow = styled.div({
  marginBottom: '12px',
  display: 'flex',
});

export const StyledFormMessageRow = styled.div({
  display: 'flex',
  flex: 1,
});

const input = {
  flex: 1,
  borderRadius: '4px',
  padding: '14px',
  color: Colors.blue,
  backgroundColor: Colors.white,
  border: 'none',
};

export const StyledEmailInput = styled.input.attrs({ type: 'email' })(smallText, input, {
  lineHeight: '26px',
  fontWeight: 400,
});

export const StyledMessageInput = styled.textarea(smallText, input, {
  resize: 'none',
  fontWeight: 400,
});

export const StyledStatusIcon = styled.div({
  display: 'flex',
  justifyContent: 'center',
  marginBottom: '32px',
});

export const StyledSentMessage = styled.span(smallText, {
  overflow: 'visible',
  color: Colors.white60,
});

export const StyledThanks = styled.span({
  color: Colors.green,
});

export const StyledEmail = styled.span({
  fontWeight: 900,
  color: Colors.white,
});

export const StyledSendStatus = styled.span(hugeText, {
  marginBottom: '4px',
});
