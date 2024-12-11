import styled from 'styled-components';

import { Colors, Spacings } from '../../tokens';
import { LabelTiny } from '../common/text';

export const CellFooter = styled.div({
  margin: `${Spacings.spacing1} ${Spacings.spacing6} 0px`,
});

export const CellFooterText = styled(LabelTiny).attrs({
  color: Colors.white60,
})({});
