import { Text, TextProps } from '../../../text';
import { useSwitchContext } from '../../';

export type SwitchLabelProps = TextProps;

export function SwitchLabel({ children, ...props }: SwitchLabelProps) {
  const { labelId, disabled } = useSwitchContext();

  return (
    <Text
      id={labelId}
      variant="bodySmallSemibold"
      color={disabled ? 'whiteAlpha40' : 'white'}
      {...props}>
      {children}
    </Text>
  );
}
