import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../foundations';
import { FlexColumn } from '../flex-column';
import { TextFieldInputGroup, TextFieldSupportingText } from './components';
import { TextFieldLabel } from './components/text-field-label';
import { TextFieldProvider } from './TextFieldContext';

export type TextFieldVariant = 'primary' | 'secondary';

export type TextFieldProps = React.ComponentPropsWithRef<'div'> & {
  invalid?: boolean;
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  variant?: TextFieldVariant;
};

export const StyledTextField = styled(FlexColumn)`
  position: relative;
  gap: ${spacings.tiny};
`;

function TextField({
  invalid,
  value,
  onValueChange,
  disabled,
  variant,
  children,
  ...props
}: TextFieldProps) {
  return (
    <TextFieldProvider
      invalid={invalid}
      value={value}
      onValueChange={onValueChange}
      disabled={disabled}
      variant={variant}>
      <StyledTextField {...props}>{children}</StyledTextField>
    </TextFieldProvider>
  );
}

const TextFieldNamespace = Object.assign(TextField, {
  Label: TextFieldLabel,
  InputGroup: TextFieldInputGroup,
  SupportingText: TextFieldSupportingText,
});

export { TextFieldNamespace as TextField };
