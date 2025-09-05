import { Text, TextProps } from '../../../typography';
import { useTextFieldContext } from '../text-field-context';

export type TextFieldLabelProps = TextProps;

export const TextFieldLabel = (props: TextFieldLabelProps) => {
  const { labelId } = useTextFieldContext();
  return <Text id={labelId} variant="labelTiny" {...props} />;
};
