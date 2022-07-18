import styled from 'styled-components';

import { colors } from '../../../config.json';
import { tinyText } from '../common-styles';

export const Footer = styled.div({
  padding: '6px 22px 0px',
});

export const FooterText = styled.span(tinyText, {
  color: colors.white60,
});

export const FooterBoldText = styled(FooterText)({
  fontWeight: 900,
});
