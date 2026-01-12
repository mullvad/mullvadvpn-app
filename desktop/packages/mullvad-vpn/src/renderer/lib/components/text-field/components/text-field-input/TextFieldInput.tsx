import React from 'react';
import styled, { css } from 'styled-components';

import {
  colors,
  fontFamilies,
  fontSizes,
  lineHeights,
  Radius,
  spacings,
} from '../../../../foundations';
import { TextFieldProps, useTextFieldContext } from '../../';

export type TextFieldInputProps = Omit<React.ComponentPropsWithRef<'input'>, 'children'>;

const variants = {
  primary: {
    padding: spacings.small,
    height: '36px',
    borderRadius: Radius.radius4,
  },
  secondary: {
    padding: `${spacings.tiny} ${spacings.small}`,
    height: '32px',
    borderRadius: Radius.radius12,
  },
};

export const StyledTextFieldInput = styled.input<{ $variant?: TextFieldProps['variant'] }>`
  ${({ $variant = 'primary' }) => {
    const variant = variants[$variant];
    return css`
      all: unset;
      box-sizing: border-box;
      height: ${variant.height};

      color: ${colors.white};
      background-color: ${colors.blue40};
      padding: ${variant.padding};
      border-radius: ${variant.borderRadius};
      font-family: ${fontFamilies['--font-family-open-sans']};
      font-size: ${fontSizes['--font-size-small']};
      line-height: ${lineHeights['--line-height-small']};
      outline: 1px solid ${colors.chalkAlpha40};
      width: 100%;

      &&::placeholder {
        color: ${colors.whiteAlpha60};
      }

      &&:disabled {
        color: ${colors.whiteAlpha20};
        background-color: ${colors.whiteOnDarkBlue5};
        outline-color: transparent;
      }

      &&:disabled::placeholder {
        color: ${colors.whiteAlpha20};
      }

      &&[aria-invalid='true'] {
        outline-color: ${colors.newRed};
      }

      &&:not(:disabled):not([aria-invalid='true']):hover {
        outline-color: ${colors.chalkAlpha80};
      }
      &&:not(:disabled):focus-visible {
        outline-width: 2px;
        outline-offset: -1px;
      }
      &&:not(:disabled):not([aria-invalid='true']):focus-visible {
        outline-color: ${colors.chalk};
      }
    `;
  }}
`;

export function TextFieldInput(props: TextFieldInputProps) {
  const { value, variant, disabled, invalid, onValueChange } = useTextFieldContext();

  const handleChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onValueChange?.(event.target.value);
    },
    [onValueChange],
  );

  return (
    <StyledTextFieldInput
      type="text"
      value={value}
      disabled={disabled}
      aria-invalid={invalid}
      onChange={handleChange}
      $variant={variant}
      {...props}
    />
  );
}
