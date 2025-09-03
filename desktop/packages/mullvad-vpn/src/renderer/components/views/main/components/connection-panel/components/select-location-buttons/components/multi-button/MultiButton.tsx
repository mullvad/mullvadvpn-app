import React from 'react';
import styled from 'styled-components';

import { Button, ButtonProps } from '../../../../../../../../../lib/components';

const ButtonRow = styled.div({
  display: 'flex',
  gap: '1px',
});

const MainButton = styled(Button)({
  borderTopRightRadius: 0,
  borderBottomRightRadius: 0,
  paddingLeft: '44px',
  '&:focus-visible': {
    zIndex: 10,
  },
});

const SideButton = styled(Button)({
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
