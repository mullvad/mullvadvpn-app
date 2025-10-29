import styled from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { useTextFieldContext } from '../../';

export type TextFieldInputProps = React.ComponentPropsWithRef<'input'>;

export const StyledTextField = styled.input`
  all: unset;
  color: ${colors.white};
  background-color: ${colors.blue40};
  padding: ${spacings.small};
  outline: 1px solid ${colors.chalkAlpha40};
  border-radius: ${Radius.radius4};
  font-family: var(--font-family-open-sans);
  font-size: 14px;
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

export function TextFieldInput(props: TextFieldInputProps) {
  const { disabled, invalid } = useTextFieldContext();

  return <StyledTextField type="text" disabled={disabled} aria-invalid={invalid} {...props} />;
}
