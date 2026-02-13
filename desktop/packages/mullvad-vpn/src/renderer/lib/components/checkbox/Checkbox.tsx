import React from 'react';

import { CheckboxProvider } from './CheckboxContext';
import { CheckboxInput, CheckboxLabel, CheckboxTrigger } from './components';

export type CheckboxProps = {
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  inputId?: string;
  descriptionId?: string;
  disabled?: boolean;
  children: React.ReactNode;
};

function Checkbox({
  inputId,
  descriptionId,
  checked,
  onCheckedChange,
  disabled,
  children,
}: CheckboxProps) {
  return (
    <CheckboxProvider
      inputId={inputId}
      descriptionId={descriptionId}
      checked={checked}
      onCheckedChange={onCheckedChange}
      disabled={disabled}>
      {children}
    </CheckboxProvider>
  );
}

const CheckboxNamespace = Object.assign(Checkbox, {
  Label: CheckboxLabel,
  Input: CheckboxInput,
  Trigger: CheckboxTrigger,
});

export { CheckboxNamespace as Checkbox };
