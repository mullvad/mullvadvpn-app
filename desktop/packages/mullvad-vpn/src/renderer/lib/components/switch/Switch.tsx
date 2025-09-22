import React from 'react';

import { SwitchLabel, SwitchThumb, SwitchTrigger } from './components';
import { SwitchProvider } from './SwitchContext';

export interface SwitchProps {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  labelId?: string;
  disabled?: boolean;
  children: React.ReactNode;
}

function Switch({
  labelId: labelIdProp,
  checked,
  onCheckedChange,
  disabled,
  children,
}: SwitchProps) {
  const labelId = React.useId();
  return (
    <SwitchProvider
      labelId={labelIdProp ?? labelId}
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
