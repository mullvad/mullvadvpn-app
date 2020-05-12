import { Styles } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { Container } from './Layout';

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
  flexDirection: 'column',
});

export const AccountContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingBottom: '48px',
});

export const AccountRow = styled.div({
  padding: '0 24px',
  marginBottom: '24px',
});

const AccountRowText = styled.span({
  display: 'block',
  fontFamily: 'Open Sans',
});

export const AccountRowLabel = styled(AccountRowText)({
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  letterSpacing: -0.2,
  marginBottom: '9px',
  color: colors.white60,
});

export const AccountRowValue = styled(AccountRowText)({
  fontSize: '16px',
  lineHeight: '19px',
  fontWeight: 800,
  color: colors.white,
});

export const AccountOutOfTime = styled(AccountRowValue)({
  color: colors.red,
});

export const AccountFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: '0 24px',
});

export default {
  button: Styles.createViewStyle({
    marginBottom: 24,
  }),
};
