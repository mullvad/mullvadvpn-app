import React from 'react';
import styled from 'styled-components';

import { ButtonProps } from './common/molecules';

const ButtonRow = styled.div({
  display: 'flex',
  gap: '1px',
});

const MainButton = styled.button({
  borderTopRightRadius: 0,
  borderBottomRightRadius: 0,
  paddingLeft: '44px',
  '&:focus-visible': {
    zIndex: 10,
  },
});

const SideButton = styled.button({
  borderTopLeftRadius: 0,
  borderBottomLeftRadius: 0,
  '&:focus-visible': {
    zIndex: 10,
  },
});

interface MultiButtonProps {
  mainButton: React.ComponentType<ButtonProps>;
  sideButton: React.ComponentType<ButtonProps>;
}

export function MultiButton(props: MultiButtonProps) {
  return (
    <ButtonRow>
      <MainButton as={props.mainButton} />
      <SideButton as={props.sideButton} />
    </ButtonRow>
  );
}
