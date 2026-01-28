import styled, { css } from 'styled-components';

import { TextField } from '../../../../../text-field';
import { TextFieldInputProps } from '../../../../../text-field/components';

type ListItemTextFieldInputWidths = 'small' | 'medium';

export type ListItemTextFieldInputProps = TextFieldInputProps & {
  width?: ListItemTextFieldInputWidths;
};

const StyledTextFieldInput = styled(TextField.Input)<{ $width: ListItemTextFieldInputWidths }>`
  ${({ $width }) => {
    return css`
      width: ${$width === 'small' ? '102px' : '206px'};
    `;
  }}
`;

export function ListItemTextFieldInput({
  width = 'medium',
  ...props
}: ListItemTextFieldInputProps) {
  return <StyledTextFieldInput $width={width} {...props} />;
}
