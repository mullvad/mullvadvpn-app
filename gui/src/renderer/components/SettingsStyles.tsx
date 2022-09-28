import styled from 'styled-components';

import { colors } from '../../config.json';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { measurements } from './common-styles';
import { NavigationScrollbars } from './NavigationBar';

export const StyledOutOfTimeSubText = styled(Cell.SubText)((props: { isOutOfTime: boolean }) => ({
  color: props.isOutOfTime ? colors.red : undefined,
}));

export const StyledCellIcon = styled(Cell.UntintedIcon)({
  marginRight: '8px',
});

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  overflow: 'visible',
});

export const StyledSettingsContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

export const StyledQuitButton = styled(AppButton.RedButton)({
  margin: measurements.viewMargin,
  marginTop: measurements.rowVerticalMargin,
});
