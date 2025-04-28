import styled from 'styled-components';

import { LabelTiny } from '../../lib/components';
import { spacings } from '../../lib/foundations';

export const CellFooter = styled.div({
  margin: `${spacings.tiny} ${spacings.large} 0px`,
});

export const CellFooterText = styled(LabelTiny).attrs({
  color: 'white60',
})({});
