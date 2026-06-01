import { FootnoteMini, type FootnoteMiniProps } from '../../../text/components';
import { useTextFieldContext } from '../../TextFieldContext';

export type TextFieldSupportingTextProps = FootnoteMiniProps;

export function TextFieldSupportingText(props: TextFieldSupportingTextProps) {
  const { invalid, descriptionId } = useTextFieldContext();
  const color = invalid ? 'red' : 'white';

  return <FootnoteMini id={descriptionId} color={color} {...props} />;
}
