import styled from 'styled-components';

import * as Cell from '../../cell';
import { ScopeBar } from './components';

export const StyledScopeBar = styled(ScopeBar)({
  marginBottom: '16px',
});

export const StyledSelectionUnavailableText = styled(Cell.CellFooterText)({
  textAlign: 'center',
});
