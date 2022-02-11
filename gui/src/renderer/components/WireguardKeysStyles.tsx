import styled from 'styled-components';
import { colors } from '../../config.json';
import { normalText, smallText, tinyText } from './common-styles';
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
  fontWeight: props.success ? 600 : 700,
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

export const StyledRowLabel = styled.span(tinyText, {
  color: colors.white60,
  lineHeight: '20px',
  marginBottom: '5px',
});

export const StyledRowLabelSpacer = styled.div({
  flex: 1,
});

export const StyledRowValue = styled.span(normalText, {
  fontWeight: 600,
  color: colors.white,
});
