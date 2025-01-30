import styled from 'styled-components';

import { Colors } from '../lib/foundations';
import { normalText, smallText, tinyText } from './common-styles';
import FormattableTextInput from './FormattableTextInput';

export const StyledLabel = styled.span(smallText, {
  color: Colors.white,
  marginBottom: '9px',
});

export const StyledInput = styled(FormattableTextInput)(normalText, {
  flex: 1,
  overflow: 'hidden',
  padding: '14px',
  fontWeight: 600,
  lineHeight: '26px',
  color: Colors.blue,
  backgroundColor: Colors.white,
  border: 'none',
  borderRadius: '4px',
  '&&::placeholder': {
    color: Colors.blue40,
  },
});

export const StyledResponse = styled.span(tinyText, {
  lineHeight: '20px',
  marginTop: '8px',
  color: Colors.white,
});

export const StyledProgressResponse = styled(StyledResponse)({
  marginTop: 0,
});

export const StyledErrorResponse = styled(StyledResponse)({
  color: Colors.red,
});

export const StyledEmptyResponse = styled.span({
  height: '20px',
  marginTop: '8px',
});

export const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
  marginTop: '25px',
});

export const StyledTitle = styled.span(smallText, {
  lineHeight: '22px',
  fontWeight: 400,
  color: Colors.white,
  marginBottom: '5px',
});
