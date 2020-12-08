import styled from 'styled-components';
import { colors } from '../../config.json';
import FormattableTextInput from './FormattableTextInput';
import ImageView from './ImageView';

export const StyledLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white,
  marginBottom: '9px',
});

export const StyledInput = styled(FormattableTextInput)({
  flex: 1,
  overflow: 'hidden',
  padding: '14px',
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '26px',
  color: colors.blue,
  backgroundColor: colors.white,
  border: 'none',
  borderRadius: '4px',
  '::placeholder': {
    color: colors.blue40,
  },
});

export const StyledResponse = styled.span({
  marginTop: '8px',
  fontFamily: 'Open Sans',
  fontSize: '13px',
  lineHeight: '20px',
});

export const StyledSuccessResponse = styled(StyledResponse)({
  fontWeight: 600,
  color: colors.green,
});

export const StyledErrorResponse = styled(StyledResponse)({
  fontWeight: 800,
  color: colors.red,
});

export const StyledEmptyResponse = styled.span({
  height: '20px',
  marginTop: '8px',
});

export const StyledSpinner = styled(ImageView)({
  marginTop: '8px',
});
