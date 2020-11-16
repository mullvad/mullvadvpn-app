import styled from 'styled-components';
import { smallText } from '../common-styles';

export const Footer = styled.div({
  padding: '6px 22px 20px',
});

export const FooterText = styled.span(smallText);

export const FooterBoldText = styled(FooterText)({
  fontWeight: 900,
});
