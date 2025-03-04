import styled from 'styled-components';

import { Spacings } from '../../../foundations';
import { Flex } from '../../flex';

export const NavigationHeaderButtonGroup = styled(Flex).attrs({
  $gap: Spacings.large,
  $alignItems: 'center',
})({});
