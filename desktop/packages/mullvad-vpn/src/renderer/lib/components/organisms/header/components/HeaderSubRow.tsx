import styled from 'styled-components';

import { Spacings } from '../../../../foundations';
import { Flex } from '../../../layout';

export const HeaderSubRow = styled(Flex).attrs({
  $flex: 1,
  $alignItems: 'flex-end',
  $gap: Spacings.spacing6,
})({
  minHeight: '18px',
});
