import styled from 'styled-components';

import { colors } from '../../../config.json';
import { spacings, tinyText } from '../common-styles';

export const CellFooter = styled.div({
  margin: `${spacings.spacing1} ${spacings.spacing6} 0px`,
});

export const CellFooterText = styled.span(tinyText, {
  color: colors.white60,
});

export const CellFooterBoldText = styled(CellFooterText)({
  fontWeight: 900,
});
