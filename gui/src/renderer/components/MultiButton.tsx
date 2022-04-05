import React from 'react';
import styled from 'styled-components';

import * as AppButton from './AppButton';

const SIDE_BUTTON_WIDTH = 50;

const ButtonRow = styled.div({
  display: 'flex',
  flexDirection: 'row',
});

const MainButton = styled.button({
  display: 'flex',
  flex: 1,
  borderTopRightRadius: 0,
  borderBottomRightRadius: 0,
});

const SideButton = styled.button({
  display: 'flex',
  borderTopLeftRadius: 0,
  borderBottomLeftRadius: 0,
  width: SIDE_BUTTON_WIDTH,
  alignItems: 'center',
  marginLeft: 1,
});

interface IMultiButtonProps {
  mainButton: React.ComponentType<AppButton.IProps>;
  sideButton: React.ComponentType<AppButton.IProps>;
}

export function MultiButton(props: IMultiButtonProps) {
  return (
    <ButtonRow>
      <MainButton as={props.mainButton} textOffset={SIDE_BUTTON_WIDTH} />
      <SideButton as={props.sideButton} />
    </ButtonRow>
  );
}
