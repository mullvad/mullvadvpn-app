import styled from 'styled-components';

import { createComponentNamespace } from '../../../../../utils';
import { spacings } from '../../../../foundations';
import {
  StyledTextFieldIcon,
  StyledTextFieldIconButton,
  StyledTextFieldInput,
  TextFieldIcon,
  TextFieldIconButton,
  TextFieldInput,
} from './components';

export type TextFieldInputGroupProps = React.PropsWithChildren;

export const StyledTextFieldInputGroup = styled.div`
  position: relative;

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

function TextFieldInputGroup({ children }: TextFieldInputGroupProps) {
  return <StyledTextFieldInputGroup>{children}</StyledTextFieldInputGroup>;
}

const TextFieldInputGroupNamespace = createComponentNamespace(TextFieldInputGroup, {
  Input: TextFieldInput,
  Icon: TextFieldIcon,
  IconButton: TextFieldIconButton,
});

export { TextFieldInputGroupNamespace as TextFieldInputGroup };
