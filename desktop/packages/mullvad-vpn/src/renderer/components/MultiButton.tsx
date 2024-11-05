import React from 'react';
import styled from 'styled-components';

const SIDE_BUTTON_WIDTH = 44;

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
  marginLeft: '1px !important',
});

export interface MultiButtonCompatibleProps {
  className?: string;
  textOffset?: number;
}

interface IMultiButtonProps {
  mainButton: React.ComponentType<MultiButtonCompatibleProps>;
  sideButton: React.ComponentType<MultiButtonCompatibleProps>;
}

export function MultiButton(props: IMultiButtonProps) {
  return (
    <ButtonRow>
      <MainButton as={props.mainButton} textOffset={SIDE_BUTTON_WIDTH + 1} />
      <SideButton as={props.sideButton} />
    </ButtonRow>
  );
}
