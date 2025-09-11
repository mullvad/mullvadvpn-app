import styled from 'styled-components';

import { TextField } from '../../../../text-field';
import { TextFieldInputProps } from '../../../../text-field/components';

export type ListItemTextFieldInputProps = TextFieldInputProps;

const StyledTextFieldInput = styled(TextField.Input)`
  box-sizing: border-box;
  width: 102px;
`;

export function ListItemTextFieldInput({ children, ...props }: ListItemTextFieldInputProps) {
  return <StyledTextFieldInput {...props}>{children}</StyledTextFieldInput>;
}
