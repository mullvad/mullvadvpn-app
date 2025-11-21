import React from 'react';

import { SwitchLabel, SwitchThumb, SwitchTrigger } from './components';
import { SwitchProvider } from './SwitchContext';

export interface SwitchProps {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  labelId?: string;
  descriptionId?: string;
  disabled?: boolean;
  children: React.ReactNode;
}

function Switch({
  labelId,
  descriptionId,
  checked,
  onCheckedChange,
  disabled,
  children,
}: SwitchProps) {
  return (
    <SwitchProvider
      labelId={labelId}
      descriptionId={descriptionId}
      checked={checked}
      onCheckedChange={onCheckedChange}
      disabled={disabled}>
      {children}
    </SwitchProvider>
  );
}

const SwitchNamespace = Object.assign(Switch, {
  Label: SwitchLabel,
  Thumb: SwitchThumb,
  Trigger: SwitchTrigger,
});

export { SwitchNamespace as Switch };
