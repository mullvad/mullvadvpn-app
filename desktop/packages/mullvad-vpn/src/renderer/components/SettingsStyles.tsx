import styled from 'styled-components';

import * as AppButton from './AppButton';
import * as Cell from './cell';
import { measurements, spacings } from './common-styles';
import { NavigationScrollbars } from './NavigationBar';

export const StyledCellIcon = styled(Cell.UntintedIcon)({
  marginRight: spacings.spacing3,
});

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  overflow: 'visible',
  marginBottom: measurements.viewMargin,
});

export const StyledSettingsContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  gap: spacings.spacing6,
});

export const StyledSettingsGroups = styled.div({
  display: 'flex',
  flexDirection: 'column',
  gap: spacings.spacing5,
});

export const StyledQuitButton = styled(AppButton.RedButton)({
  margin: `0 ${measurements.viewMargin}`,
});
