import styled from 'styled-components';

import { LabelTiny } from '../../lib/components';
import { DeprecatedColors, spacings } from '../../lib/foundations';

export const CellFooter = styled.div({
  margin: `${spacings.tiny} ${spacings.large} 0px`,
});

export const CellFooterText = styled(LabelTiny).attrs({
  color: DeprecatedColors.white60,
})({});
