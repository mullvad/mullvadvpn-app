import styled from 'styled-components';

import { TextField } from '../../../../../text-field';
import { TextFieldInputProps } from '../../../../../text-field/components';

export type ListItemTextFieldInputProps = TextFieldInputProps;

const StyledTextFieldInput = styled(TextField.Input)`
  width: 102px;
`;

export function ListItemTextFieldInput(props: ListItemTextFieldInputProps) {
  return <StyledTextFieldInput {...props} />;
}
