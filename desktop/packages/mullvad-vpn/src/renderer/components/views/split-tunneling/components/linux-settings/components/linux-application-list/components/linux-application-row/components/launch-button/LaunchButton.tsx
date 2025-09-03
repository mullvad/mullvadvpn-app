import React from 'react';

import { useDisabled, useLaunchApplication } from '../../hooks';
import { StyledCellButton } from './LaunchButtonStyles';

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
