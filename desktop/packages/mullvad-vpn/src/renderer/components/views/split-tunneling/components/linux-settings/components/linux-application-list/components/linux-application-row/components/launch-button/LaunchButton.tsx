import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../../../../../../../../lib/foundations';
import { CellButton } from '../../../../../../../../../../cell';
import { useDisabled, useLaunchApplication } from '../../hooks';

export const StyledCellButton = styled(CellButton)<{ $lookDisabled?: boolean }>((props) => ({
  '&&:not(:disabled):hover': {
    backgroundColor: props.$lookDisabled ? colors.blue : undefined,
  },
}));

export type LaunchButtonProps = {
  children: React.ReactNode;
};

export function LaunchButton({ children }: LaunchButtonProps) {
  const disabled = useDisabled();
  const launchApplication = useLaunchApplication();

  return (
    <StyledCellButton onClick={launchApplication} $lookDisabled={disabled}>
      {children}
    </StyledCellButton>
  );
}
