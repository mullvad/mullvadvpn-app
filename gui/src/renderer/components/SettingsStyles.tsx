import styled from 'styled-components';
import { colors } from '../../config.json';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { Container } from './Layout';
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

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  justifyContent: 'space-between',
  overflow: 'visible',
});

export const StyledCellSpacer = styled.div({
  height: '20px',
  minHeight: '20px',
  flex: 0,
});

export const StyledQuitButton = styled(AppButton.RedButton)({
  margin: '20px 22px 22px',
});
