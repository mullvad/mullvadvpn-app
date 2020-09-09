import styled from 'styled-components';
import { colors } from '../../config.json';
import { smallText } from './common-styles';
import { Container } from './Layout';
import { NavigationScrollbars } from './NavigationBar';

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const StyledMessages = styled.div({
  padding: '0 22px 20px',
  flex: 1,
});

export const StyledMessage = styled.span(smallText, (props: { success: boolean }) => ({
  fontWeight: props.success ? 600 : 800,
  color: props.success ? colors.green : colors.red,
}));

export const StyledRow = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: '0 22px',
  marginBottom: '20px',
});

export const StyledButtonRow = styled(StyledRow)({
  marginBottom: '18px',
});

export const StyledLastButtonRow = styled(StyledButtonRow)({
  marginBottom: '22px',
});

export const StyledRowLabel = styled.span(smallText, {
  display: 'flex',
  color: colors.white60,
  marginBottom: '9px',
});

export const StyledRowLabelSpacer = styled.div({
  flex: 1,
});

export const StyledRowValue = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '16px',
  lineHeight: '19px',
  fontWeight: 800,
  color: colors.white,
});
