import styled from 'styled-components';

import { LabelTiny } from '../../lib/components';
import { Colors, Spacings } from '../../lib/foundations';

export const CellFooter = styled.div({
  margin: `${Spacings.spacing1} ${Spacings.spacing6} 0px`,
});

export const CellFooterText = styled(LabelTiny).attrs({
  color: Colors.white60,
})({});
