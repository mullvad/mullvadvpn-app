import styled from 'styled-components';

import { FootnoteMini, type FootnoteMiniProps } from '../../../text/components';
import { useTextFieldContext } from '../../TextFieldContext';

export type TextFieldSupportingTextProps = FootnoteMiniProps;

export const StyledTextFieldSupportingText = styled(FootnoteMini)`
  min-width: 100%;
`;

export function TextFieldSupportingText(props: TextFieldSupportingTextProps) {
  const { invalid, descriptionId } = useTextFieldContext();
  const color = invalid ? 'red' : 'white';

  return <StyledTextFieldSupportingText id={descriptionId} color={color} {...props} />;
}
