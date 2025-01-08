import styled from 'styled-components';

import { Spacings } from '../../../../foundations';
import { Flex } from '../../../layout';

export const NavigationHeaderButtonGroup = styled(Flex).attrs({
  $gap: Spacings.spacing6,
  $alignItems: 'center',
})({});
