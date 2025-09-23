import { Text, TextProps } from '../../../text';
import { useTextFieldContext } from '../../';

export type TextFieldLabelProps = TextProps;

export const TextFieldLabel = (props: TextFieldLabelProps) => {
  const { labelId } = useTextFieldContext();
  return <Text id={labelId} variant="labelTinySemiBold" {...props} />;
};
