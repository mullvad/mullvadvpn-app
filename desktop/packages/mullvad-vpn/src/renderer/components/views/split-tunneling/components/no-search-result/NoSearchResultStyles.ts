import styled from 'styled-components';

import { spacings } from '../../../../../lib/foundations';
import { CellFooter, CellFooterText } from '../../../../cell';

export const StyledNoResult = styled(CellFooter)({
  display: 'flex',
  flexDirection: 'column',
  paddingTop: 0,
  marginTop: 0,
  marginBottom: spacings.large,
});

export const StyledNoResultText = styled(CellFooterText)({
  textAlign: 'center',
});
