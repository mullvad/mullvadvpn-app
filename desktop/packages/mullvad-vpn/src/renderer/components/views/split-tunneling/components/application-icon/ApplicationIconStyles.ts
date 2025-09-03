import styled from 'styled-components';

import { spacings } from '../../../../../lib/foundations';
import { CellImage } from '../../../../cell';
import { disabledApplication, type DisabledApplicationProps } from '../../utils';

export const StyledIcon = styled(CellImage)<DisabledApplicationProps>(disabledApplication, {
  marginRight: spacings.small,
});

export const StyledIconPlaceholder = styled.div({
  width: '35px',
  marginRight: spacings.small,
});
