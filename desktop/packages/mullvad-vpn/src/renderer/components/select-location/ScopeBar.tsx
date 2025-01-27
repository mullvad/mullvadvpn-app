import React, { useCallback } from 'react';
import styled from 'styled-components';

import { Colors } from '../../lib/foundations';
import { smallText } from '../common-styles';

const StyledScopeBar = styled.div({
  display: 'flex',
  flexDirection: 'row',
  backgroundColor: Colors.blue40,
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

const StyledScopeBarItem = styled.button<{ selected?: boolean }>(smallText, (props) => ({
  cursor: 'default',
  flex: 1,
  flexBasis: 0,
  padding: '4px 8px',
  color: Colors.white,
  textAlign: 'center',
  border: 'none',
  backgroundColor: props.selected ? Colors.green : 'transparent',
  '&&:hover': {
    backgroundColor: props.selected ? Colors.green : Colors.blue40,
  },
}));

interface IScopeBarItemProps {
  index?: number;
  selected?: boolean;
  onClick?: (index: number) => void;
  children?: React.ReactNode;
}

export function ScopeBarItem(props: IScopeBarItemProps) {
  const { onClick: propOnClick } = props;

  const onClick = useCallback(() => {
    if (props.index !== undefined) {
      propOnClick?.(props.index);
    }
  }, [propOnClick, props.index]);

  return props.index !== undefined ? (
    <StyledScopeBarItem selected={props.selected} onClick={onClick}>
      {props.children}
    </StyledScopeBarItem>
  ) : null;
}
