import styled from 'styled-components';

import { measurements } from '../../common-styles';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SearchBar from '../../SearchBar';

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

export const StyledSearchBar = styled(SearchBar)({
  marginLeft: measurements.horizontalViewMargin,
  marginRight: measurements.horizontalViewMargin,
  marginBottom: measurements.buttonVerticalMargin,
});
