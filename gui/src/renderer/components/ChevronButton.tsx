import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './Cell';

interface IProps {
  up: boolean;
  onClick?: (event: React.MouseEvent) => void;
  className?: string;
}

const Icon = styled(Cell.Icon)({
  flex: 0,
  alignSelf: 'stretch',
  justifyContent: 'center',
});

export default function ChevronButton(props: IProps) {
  return (
    <Icon
      tintColor={colors.white80}
      tintHoverColor={colors.white}
      onClick={props.onClick}
      source={props.up ? 'icon-chevron-up' : 'icon-chevron-down'}
      height={24}
      width={24}
      className={props.className}
    />
  );
}
