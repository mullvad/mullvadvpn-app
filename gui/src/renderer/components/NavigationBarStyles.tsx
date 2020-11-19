import styled from 'styled-components';
import { colors } from '../../config.json';
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
  display: 'flex',
  flex: 1,
  flexDirection: 'row',
});

export const StyledNavigationBar = styled.nav((props: { unpinnedWindow: boolean }) => ({
  flex: 0,
  padding: '12px',
  paddingTop: process.platform === 'darwin' && !props.unpinnedWindow ? '24px' : '12px',
}));

export const StyledNavigationBarWrapper = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'column',
  overflow: 'hidden',
});

export const StyledTitleBarItemContainer = styled.div({
  display: 'flex',
  flex: 1,
  minWidth: 0,
  flexDirection: 'column',
  justifyContent: 'center',
  overflow: 'hidden',
});

interface ITitleBarItemLabelProps {
  titleAdjustment: number;
  visible?: boolean;
}

export const StyledTitleBarItemLabel = styled.h1({}, (props: ITitleBarItemLabelProps) => ({
  fontFamily: 'Open Sans',
  fontSize: '16px',
  fontWeight: 600,
  lineHeight: '22px',
  color: colors.white,
  padding: '0 5px',
  textAlign: 'center',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
  marginLeft: props.titleAdjustment + 'px',
  opacity: props.visible ? 1 : 0,
  transition: 'opacity 250ms ease-in-out',
}));

export const StyledTitleBarItemMeasuringLabel = styled(StyledTitleBarItemLabel)({
  position: 'absolute',
  opacity: 0,
});

export const StyledCloseBarItemButton = styled.button({
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
});

export const StyledCloseBarItemIcon = styled(ImageView)({
  flex: 0,
});

export const StyledBackBarItemButton = styled.button({
  position: 'relative',
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  display: 'flex',
  flexDirection: 'row',
  alignItems: 'center',
  backgroundColor: 'transparent',
  zIndex: 1,
});

export const StyledBackBarItemIcon = styled(ImageView)({
  marginRight: '8px',
  [StyledBackBarItemButton + ':hover &']: {
    backgroundColor: colors.white80,
  },
});

export const StyledBackBarItemLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  color: colors.white60,
  whiteSpace: 'nowrap',
  [StyledBackBarItemButton + ':hover &']: {
    color: colors.white80,
  },
});
