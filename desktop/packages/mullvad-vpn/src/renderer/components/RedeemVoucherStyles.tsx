import styled from 'styled-components';

import { DeprecatedColors } from '../lib/foundations';
import { normalText, smallText, tinyText } from './common-styles';
import FormattableTextInput from './FormattableTextInput';

export const StyledLabel = styled.span(smallText, {
  color: DeprecatedColors.white,
  marginBottom: '9px',
});

export const StyledInput = styled(FormattableTextInput)(normalText, {
  flex: 1,
  overflow: 'hidden',
  padding: '14px',
  fontWeight: 600,
  lineHeight: '26px',
  color: DeprecatedColors.blue,
  backgroundColor: DeprecatedColors.white,
  border: 'none',
  borderRadius: '4px',
  '&&::placeholder': {
    color: DeprecatedColors.blue40,
  },
});

export const StyledResponse = styled.span(tinyText, {
  lineHeight: '20px',
  marginTop: '8px',
  color: DeprecatedColors.white,
});

export const StyledProgressResponse = styled(StyledResponse)({
  marginTop: 0,
});

export const StyledErrorResponse = styled(StyledResponse)({
  color: DeprecatedColors.red,
});

export const StyledEmptyResponse = styled.span({
  height: '20px',
  marginTop: '8px',
});

export const StyledTitle = styled.span(smallText, {
  lineHeight: '22px',
  fontWeight: 400,
  color: DeprecatedColors.white,
  marginBottom: '5px',
});
