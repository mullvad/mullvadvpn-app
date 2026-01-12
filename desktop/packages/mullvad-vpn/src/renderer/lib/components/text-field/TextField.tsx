import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../foundations';
import {
  StyledTextFieldIcon,
  StyledTextFieldIconButton,
  StyledTextFieldInput,
  TextFieldIcon,
  TextFieldIconButton,
  TextFieldInput,
  TextFieldLabel,
} from './components';
import { TextFieldProvider } from './TextFieldContext';

export type TextFieldVariant = 'primary' | 'secondary';

export type TextFieldProps = React.PropsWithChildren<{
  invalid?: boolean;
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  variant?: TextFieldVariant;
}>;

export const StyledTextField = styled.div`
  position: relative;
  display: flex;
  flex-grow: 1;

  // If contains an Icon followed by an Input, add padding to the input
  &&:has(> ${StyledTextFieldIcon} + ${StyledTextFieldInput}) {
    ${StyledTextFieldInput} {
      // Icon size is 18px
      padding-left: calc(${spacings.small} + 18px + ${spacings.tiny});
    }
  }

  // If contains an Input followed by an IconButton, add padding to the input
  &&:has(> ${StyledTextFieldInput} + ${StyledTextFieldIconButton}) {
    ${StyledTextFieldInput} {
      // Icon size is 18px
      padding-right: calc(${spacings.small} + 18px + ${spacings.tiny});
    }
  }
`;

function TextField({ children, ...props }: TextFieldProps) {
  return (
    <TextFieldProvider {...props}>
      <StyledTextField>{children}</StyledTextField>
    </TextFieldProvider>
  );
}

const TextFieldNamespace = Object.assign(TextField, {
  Input: TextFieldInput,
  Label: TextFieldLabel,
  Icon: TextFieldIcon,
  IconButton: TextFieldIconButton,
});

export { TextFieldNamespace as TextField };
