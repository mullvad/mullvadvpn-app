import React, { useCallback, useContext, useEffect, useMemo } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { useBoolean, useStyledRef } from '../lib/utilityHooks';
import { smallText } from './common-styles';
import { BackAction } from './KeyboardNavigation';

const BORDER_WIDTH = 1;
const PADDING_VERTICAL = 10;
const ITEM_HEIGHT = 22;

type Alignment = 'left' | 'right';
type Direction = 'up' | 'down';

interface MenuContext {
  getTriggerBounds: () => DOMRect;
  toggleVisibility: () => void;
  hide: () => void;
  visible: boolean;
}

const menuContext = React.createContext<MenuContext>({
  getTriggerBounds: () => {
    throw new Error('No trigger bounds available');
  },
  toggleVisibility: () => {
    throw new Error('toggleVisibility not defined');
  },
  hide: () => {
    throw new Error('hide not defined');
  },
  visible: false,
});

const StyledMenuContainer = styled.div({
  position: 'relative',
  padding: '8px 4px',
});

export function ContextMenuContainer(props: React.PropsWithChildren) {
  const ref = useStyledRef<HTMLDivElement>();
  const [visible, , hide, toggleVisibility] = useBoolean(false);

  const getTriggerBounds = useCallback(() => {
    if (ref.current === null) {
      throw new Error('No trigger bounds available');
    }
    return ref.current.getBoundingClientRect();
  }, [ref.current]);

  const contextValue = useMemo(
    () => ({
      getTriggerBounds,
      toggleVisibility,
      visible,
      hide,
    }),
    [getTriggerBounds, visible],
  );

  const clickOutsideListener = useCallback(
    (event: MouseEvent) => {
      if (
        visible &&
        event.target !== null &&
        ref.current?.contains(event.target as HTMLElement) === false
      ) {
        hide();
      }
    },
    [visible],
  );

  useEffect(() => {
    document.addEventListener('click', clickOutsideListener, true);
    return () => document.removeEventListener('click', clickOutsideListener, true);
  }, [clickOutsideListener]);

  return (
    <StyledMenuContainer ref={ref}>
      <menuContext.Provider value={contextValue}>{props.children}</menuContext.Provider>
    </StyledMenuContainer>
  );
}

const StyledTrigger = styled.button({
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
});

export function ContextMenuTrigger(props: React.PropsWithChildren) {
  const { toggleVisibility } = useContext(menuContext);

  return <StyledTrigger onClick={toggleVisibility}>{props.children}</StyledTrigger>;
}

interface StyledMenuProps {
  $direction: Direction;
  $align: Alignment;
}

const StyledMenu = styled.div<StyledMenuProps>((props) => {
  const oppositeSide = 'calc(100% - 8px)';
  const iconMargin = '12px';

  return {
    position: 'absolute',
    top: props.$direction === 'up' ? 'auto' : oppositeSide,
    bottom: props.$direction === 'up' ? oppositeSide : 'auto',
    left: props.$align === 'left' ? iconMargin : 'auto',
    right: props.$align === 'left' ? 'auto' : iconMargin,
    padding: '7px 4px',
    background: 'rgb(36, 53, 78)',
    border: `1px solid ${colors.darkBlue}`,
    borderRadius: '8px',
    zIndex: 1,
  };
});

const StyledMenuItem = styled.button(smallText, (props) => ({
  minWidth: '110px',
  padding: '1px 10px 2px',
  lineHeight: `${ITEM_HEIGHT}px`,
  background: 'transparent',
  border: 'none',
  textAlign: 'left',
  color: props.disabled ? colors.white50 : colors.white,

  '&&:hover': {
    background: props.disabled ? 'transparent' : colors.blue,
  },
}));

const StyledSeparator = styled.hr({
  height: '1px',
  border: 'none',
  backgroundColor: colors.darkBlue,
  margin: '4px 9px',
});

type ContextMenuItemItem = {
  type: 'item';
  label: string;
  disabled?: boolean;
  onClick: () => void;
};

type ContextMenuSeparator = { type: 'separator' };

export type ContextMenuItem = ContextMenuItemItem | ContextMenuSeparator;

interface MenuProps {
  items: Array<ContextMenuItem>;
  align: Alignment;
}

export function ContextMenu(props: MenuProps) {
  const { getTriggerBounds, visible, hide } = useContext(menuContext);

  if (!visible) {
    return null;
  }

  const triggerBounds = getTriggerBounds();
  const direction = calculateDirection(visible, triggerBounds, props.items.length);

  return (
    <BackAction action={hide}>
      <StyledMenu $direction={direction} $align={props.align}>
        {props.items.map((item, i) =>
          item.type === 'separator' ? (
            <StyledSeparator key={`separator-${i}`} />
          ) : (
            <ContextMenuItemRow key={item.label} item={item} closeMenu={hide} />
          ),
        )}
      </StyledMenu>
    </BackAction>
  );
}

function calculateDirection(
  visible: boolean,
  triggerBounds: DOMRect,
  itemsLength: number,
): Direction {
  if (visible) {
    const extraSpace = 2 * (BORDER_WIDTH + PADDING_VERTICAL);
    const downwardsStartPosition = triggerBounds.y + triggerBounds.height;
    const downwardsEndPosition = downwardsStartPosition + itemsLength * ITEM_HEIGHT + extraSpace;
    return downwardsEndPosition < window.innerHeight ? 'down' : 'up';
  } else {
    return 'down';
  }
}

interface ContextMenuItemRowProps {
  item: ContextMenuItemItem;
  closeMenu: () => void;
}

function ContextMenuItemRow(props: ContextMenuItemRowProps) {
  const onClick = useCallback(() => {
    if (!props.item.disabled) {
      props.closeMenu();
      props.item.onClick();
    }
  }, [props.closeMenu, props.item.disabled, props.item.onClick]);

  return (
    <StyledMenuItem onClick={onClick} disabled={props.item.disabled}>
      {props.item.label}
    </StyledMenuItem>
  );
}
