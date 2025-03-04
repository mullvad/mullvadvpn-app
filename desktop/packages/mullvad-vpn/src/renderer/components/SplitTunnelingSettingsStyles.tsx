import styled from 'styled-components';

import { Colors, spacings } from '../lib/foundations';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { measurements, normalText } from './common-styles';
import { NavigationScrollbars } from './NavigationScrollbars';
import SearchBar from './SearchBar';
import { SmallButton } from './SmallButton';

export const StyledPageCover = styled.div<{ $show: boolean }>((props) => ({
  position: 'absolute',
  zIndex: 2,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  opacity: 0.5,
  display: props.$show ? 'block' : 'none',
}));

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledCellButton = styled(Cell.CellButton)<{ $lookDisabled?: boolean }>((props) => ({
  '&&:not(:disabled):hover': {
    backgroundColor: props.$lookDisabled ? Colors.blue : undefined,
  },
}));

interface DisabledApplicationProps {
  $lookDisabled?: boolean;
}

const disabledApplication = (props: DisabledApplicationProps) => ({
  opacity: props.$lookDisabled ? 0.6 : undefined,
});

export const StyledIcon = styled(Cell.CellImage)<DisabledApplicationProps>(disabledApplication, {
  marginRight: spacings.small,
});

export const StyledCellWarningIcon = styled(Cell.CellTintedIcon)({
  marginLeft: spacings.small,
  marginRight: spacings.tiny,
});

export const StyledCellLabel = styled(Cell.Label)<DisabledApplicationProps>(
  disabledApplication,
  normalText,
  {
    fontWeight: 400,
    wordWrap: 'break-word',
    overflow: 'hidden',
  },
);

export const StyledIconPlaceholder = styled.div({
  width: '35px',
  marginRight: spacings.small,
});

export const StyledSpinnerRow = styled(Cell.CellButton)({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  padding: `${spacings.small} 0`,
  marginBottom: measurements.rowVerticalMargin,
  background: Colors.blue40,
});

export const StyledBrowseButton = styled(AppButton.BlueButton)({
  margin: `0 ${measurements.horizontalViewMargin} ${measurements.verticalViewMargin}`,
});

export const StyledNoResult = styled(Cell.CellFooter)({
  display: 'flex',
  flexDirection: 'column',
  paddingTop: 0,
  marginTop: 0,
  marginBottom: spacings.large,
});

export const StyledNoResultText = styled(Cell.CellFooterText)({
  textAlign: 'center',
});

export const StyledSearchBar = styled(SearchBar)({
  marginLeft: measurements.horizontalViewMargin,
  marginRight: measurements.horizontalViewMargin,
  marginBottom: measurements.buttonVerticalMargin,
});

export const WideSmallButton = styled(SmallButton)({
  width: '100%',
});
