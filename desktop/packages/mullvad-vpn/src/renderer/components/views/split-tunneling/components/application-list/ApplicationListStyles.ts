import styled from 'styled-components';

import { colors, spacings } from '../../../../../lib/foundations';
import { CellButton } from '../../../../cell';
import { measurements } from '../../../../common-styles';

export const StyledSpinnerRow = styled(CellButton)({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: `${spacings.small} 0`,
  marginBottom: measurements.rowVerticalMargin,
  background: colors.blue40,
});
