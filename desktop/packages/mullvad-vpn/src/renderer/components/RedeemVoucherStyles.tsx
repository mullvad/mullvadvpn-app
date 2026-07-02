import styled from 'styled-components';

import { colors } from '../lib/foundations';
import { normalText } from './common-styles';
import FormattableTextInput from './FormattableTextInput';

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

export const StyledEmptyResponse = styled.span({
  height: '18px',
});
