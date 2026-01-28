import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../lib/foundations';
import type { IScopeBarItemProps } from '../scope-bar-item';

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
