import styled from 'styled-components';

import { spacings } from '../../tokens';
import { LabelTiny } from '../common/text';

export const CellFooter = styled.div({
  margin: `${spacings.spacing1} ${spacings.spacing6} 0px`,
});

export const CellFooterText = styled(LabelTiny).attrs({
  $color: 'white60',
})({});
