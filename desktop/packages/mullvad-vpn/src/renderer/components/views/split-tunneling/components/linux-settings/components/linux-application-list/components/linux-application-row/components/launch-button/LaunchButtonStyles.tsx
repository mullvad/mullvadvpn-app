import styled from 'styled-components';

import { colors } from '../../../../../../../../../../../lib/foundations';
import { CellButton } from '../../../../../../../../../../cell';

export const StyledCellButton = styled(CellButton)<{ $lookDisabled?: boolean }>((props) => ({
  '&&:not(:disabled):hover': {
    backgroundColor: props.$lookDisabled ? colors.blue : undefined,
  },
}));
