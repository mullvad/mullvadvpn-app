import React from 'react';

import { SwitchProvider, SwitchThumb, SwitchTrigger } from './components';
import { SwitchLabel } from './components/switch-label';

export interface SwitchProps {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
  children: React.ReactNode;
}

function Switch({ checked, onCheckedChange, disabled, children }: SwitchProps) {
  const labelId = React.useId();
  return (
    <SwitchProvider
      labelId={labelId}
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
