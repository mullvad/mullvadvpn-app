import styled from 'styled-components';

import * as AppButton from './AppButton';
import * as Cell from './cell';
import { measurements, spacings } from './common-styles';

export const StyledCellIcon = styled(Cell.UntintedIcon)({
  marginRight: spacings.spacing3,
});

export const StyledQuitButton = styled(AppButton.RedButton)({
  margin: `0 ${measurements.horizontalViewMargin}`,
});
