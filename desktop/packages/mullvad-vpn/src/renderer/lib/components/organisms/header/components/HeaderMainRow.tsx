import styled from 'styled-components';

import { Flex } from '../../../layout';

export const HeaderMainRow = styled(Flex).attrs({
  $justifyContent: 'space-between',
  $alignItems: '  center',
})({
  minHeight: '38px',
  height: '38px',
});
