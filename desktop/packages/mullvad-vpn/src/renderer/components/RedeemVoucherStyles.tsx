import styled from 'styled-components';

import { LabelTinySemiBold } from '../lib/components';
import { colors } from '../lib/foundations';
import { normalText, smallText, tinyText } from './common-styles';
import FormattableTextInput from './FormattableTextInput';

export const StyledLabel = styled.span(smallText, {
  color: colors.white,
  marginBottom: '9px',
});

export const StyledInput = styled(FormattableTextInput)(normalText, {
  flex: 1,
  overflow: 'hidden',
  padding: '14px',
  fontWeight: 600,
  lineHeight: '26px',
  color: colors.blue,
  backgroundColor: colors.white,
  border: 'none',
  borderRadius: '4px',
  '&&::placeholder': {
    color: colors.whiteOnBlue60,
  },
});

export const StyledResponse = styled.span(tinyText, {
  lineHeight: '20px',
  marginTop: '8px',
  color: colors.white,
});

export const StyledProgressResponse = styled(StyledResponse)({
  marginTop: 0,
});

export const StyledErrorResponse = styled(StyledResponse)({
  color: colors.red,
});

export const StyledEmptyResponse = styled.span({
  height: '20px',
  marginTop: '8px',
});

export const StyledTitle = styled.span(smallText, {
  lineHeight: '22px',
  fontWeight: 400,
  color: colors.white,
  marginBottom: '5px',
});

export const StyledAccountNumberInfo = styled(LabelTinySemiBold)({
  marginTop: '8px',
});
