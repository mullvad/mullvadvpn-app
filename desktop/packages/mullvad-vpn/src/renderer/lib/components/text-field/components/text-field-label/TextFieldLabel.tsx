import { Text, TextProps } from '../../../text';
import { useTextFieldContext } from '../../';

export type TextFieldLabelProps = TextProps;

export function TextFieldLabel(props: TextFieldLabelProps) {
  const { labelId } = useTextFieldContext();
  return <Text id={labelId} variant="labelTinySemiBold" {...props} />;
}
