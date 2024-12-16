import styled from 'styled-components';

import { LabelTiny } from '../common/text';
import { Colors, Spacings } from '../common/variables';

export const CellFooter = styled.div({
  margin: `${Spacings.spacing1} ${Spacings.spacing6} 0px`,
});

export const CellFooterText = styled(LabelTiny).attrs({
  color: Colors.white60,
})({});
