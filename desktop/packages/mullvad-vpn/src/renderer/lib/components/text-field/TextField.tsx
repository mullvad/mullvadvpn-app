import React from 'react';

import { FlexColumn } from '../flex-column';
import { TextFieldInput, TextFieldLabel } from './components';
import { TextFieldProvider } from './TextFieldContext';

export type TextFieldProps = React.PropsWithChildren<{
  invalid?: boolean;
  value?: string;
  disabled?: boolean;
}>;

function TextField({ children, ...props }: TextFieldProps) {
  const labelId = React.useId();
  return (
    <TextFieldProvider labelId={labelId} {...props}>
      <FlexColumn gap="tiny">{children}</FlexColumn>
    </TextFieldProvider>
  );
}

const TextFieldNamespace = Object.assign(TextField, {
  Input: TextFieldInput,
  Label: TextFieldLabel,
});

export { TextFieldNamespace as TextField };
