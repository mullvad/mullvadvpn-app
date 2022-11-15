import React, { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { smallText } from '../common-styles';

const StyledScopeBar = styled.div({
  display: 'flex',
  flexDirection: 'row',
  backgroundColor: colors.blue40,
  borderRadius: '13px',
  overflow: 'hidden',
});

interface IScopeBarProps {
  selectedIndex: number;
  onChange?: (selectedIndex: number) => void;
  className?: string;
  children: React.ReactElement<IScopeBarItemProps>[];
}

export function ScopeBar(props: IScopeBarProps) {
  const children = React.Children.map(props.children, (child, index) => {
    if (React.isValidElement(child)) {
      return React.cloneElement(child, {
        selected: index === props.selectedIndex,
        onClick: props.onChange,
        index,
      });
    } else {
      return undefined;
    }
  });

  return <StyledScopeBar className={props.className}>{children}</StyledScopeBar>;
}

const StyledScopeBarItem = styled.button(smallText, (props: { selected?: boolean }) => ({
  cursor: 'default',
  flex: 1,
  flexBasis: 0,
  padding: '4px 8px',
  color: colors.white,
  textAlign: 'center',
  border: 'none',
  backgroundColor: props.selected ? colors.green : 'transparent',
  ':hover': {
    backgroundColor: props.selected ? colors.green : colors.blue40,
  },
}));

interface IScopeBarItemProps {
  index?: number;
  selected?: boolean;
  onClick?: (index: number) => void;
  children?: React.ReactNode;
}

export function ScopeBarItem(props: IScopeBarItemProps) {
  const onClick = useCallback(() => {
    if (props.index !== undefined) {
      props.onClick?.(props.index);
    }
  }, [props.onClick, props.index]);

  return props.index !== undefined ? (
    <StyledScopeBarItem selected={props.selected} onClick={onClick}>
      {props.children}
    </StyledScopeBarItem>
  ) : null;
}
