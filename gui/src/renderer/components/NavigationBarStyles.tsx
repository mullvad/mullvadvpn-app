import styled from 'styled-components';

import { colors } from '../../config.json';
import { normalText } from './common-styles';
import ImageView from './ImageView';

export const StyledNavigationBarSeparator = styled.div({
  backgroundColor: 'rgba(0, 0, 0, 0.2)',
  position: 'absolute',
  bottom: 0,
  left: 0,
  right: 0,
  height: '1px',
});

export const StyledNavigationItems = styled.div({
  flex: 1,
  display: 'grid',
  gridTemplateColumns: '1fr auto 1fr',
  alignItems: 'center',
});

export const StyledNavigationBar = styled.nav({
  flex: 0,
  padding: '12px',
});

export const StyledTitleBarItemLabel = styled.h1(normalText, (props: { visible?: boolean }) => ({
  fontWeight: 400,
  lineHeight: '22px',
  color: colors.white,
  padding: '0 5px',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  opacity: props.visible ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
}));

export const StyledBackBarItemButton = styled.button({
  justifySelf: 'start',
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  display: 'flex',
  flexDirection: 'row',
  alignItems: 'center',
  backgroundColor: 'transparent',
});

export const StyledBackBarItemIcon = styled(ImageView)({
  marginRight: '8px',
  [StyledBackBarItemButton + ':hover &']: {
    backgroundColor: colors.white60,
  },
});
