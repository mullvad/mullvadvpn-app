import React from 'react';

import { SwitchInput, SwitchLabel, SwitchTrigger } from './components';
import { SwitchProvider } from './SwitchContext';

export type SwitchProps = {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  inputId?: string;
  descriptionId?: string;
  disabled?: boolean;
  children: React.ReactNode;
};

function Switch({
  inputId,
  descriptionId,
  checked,
  onCheckedChange,
  disabled,
  children,
}: SwitchProps) {
  return (
    <SwitchProvider
      inputId={inputId}
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
  Input: SwitchInput,
  Trigger: SwitchTrigger,
});

export { SwitchNamespace as Switch };
