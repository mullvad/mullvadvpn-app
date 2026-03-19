import React, { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../lib/foundations';
import { smallText } from '../../../../common-styles';

const StyledScopeBarItem = styled.button<{ selected?: boolean }>(smallText, (props) => ({
  cursor: 'default',
  flex: 1,
  flexBasis: 0,
  padding: '4px 8px',
  color: colors.white,
  textAlign: 'center',
  border: 'none',
  backgroundColor: props.selected ? colors.green : colors.transparent,
  '&&:hover': {
    backgroundColor: props.selected ? colors.green : colors.blue40,
  },
}));

export interface IScopeBarItemProps {
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
