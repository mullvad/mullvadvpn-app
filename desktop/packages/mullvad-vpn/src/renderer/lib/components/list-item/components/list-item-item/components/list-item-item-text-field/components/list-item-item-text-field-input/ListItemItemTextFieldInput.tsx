import styled, { css } from 'styled-components';

import { TextField } from '../../../../../../../text-field';

type ListItemItemTextFieldInputWidths = 'small' | 'medium';

export type ListItemItemTextFieldInputProps = React.CustomComponentPropsWithRef<
  typeof TextField.Input
> & {
  width?: ListItemItemTextFieldInputWidths;
};

const StyledListItemItemTextFieldInput = styled(TextField.Input)<{
  $width: ListItemItemTextFieldInputWidths;
}>`
  ${({ $width }) => {
    return css`
      width: ${$width === 'small' ? '102px' : '206px'};
    `;
  }}
`;

export function ListItemItemTextFieldInput({
  width = 'medium',
  ...props
}: ListItemItemTextFieldInputProps) {
  return <StyledListItemItemTextFieldInput $width={width} {...props} />;
}
