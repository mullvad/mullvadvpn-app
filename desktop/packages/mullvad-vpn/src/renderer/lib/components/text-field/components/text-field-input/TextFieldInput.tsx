import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { useTextFieldContext } from '../text-field-context/TextFieldContext';

export type TextFieldInputProps = React.ComponentPropsWithRef<'input'>;

export const StyledTextField = styled.input<{ $disabled?: boolean; $invalid?: boolean }>`
  ${({ $invalid, $disabled }) => {
    // TODO: Add color to tokens
    const borderColor = $invalid ? 'rgba(235, 93, 64, 1)' : colors.chalkAlpha40;
    const backgroundColor = $disabled ? colors.whiteOnDarkBlue5 : colors.blue40;
    const color = $disabled ? colors.whiteAlpha20 : colors.white;
    return css`
      all: unset;
      background-color: ${backgroundColor};
      padding: ${spacings.small};
      border: 1px solid ${colors.whiteAlpha60};
      border-color: ${borderColor};
      font-size: 14px;
      border-radius: ${Radius.radius4};
      color: ${color};
      width: 100%;
      &&:not(:disabled):not([aria-invalid='true']):hover {
        border-color: ${colors.chalkAlpha80};
      }
      &&:not(:disabled):not([aria-invalid='true']):focus-visible {
        border-color: ${colors.chalk};
      }
    `;
  }}
`;

export function TextFieldInput(props: TextFieldInputProps) {
  const { disabled, invalid } = useTextFieldContext();

  return (
    <StyledTextField
      type="text"
      $disabled={disabled}
      disabled={disabled}
      $invalid={invalid}
      aria-invalid={invalid}
      {...props}
    />
  );
}
