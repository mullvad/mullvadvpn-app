import { Text, TextProps } from '../../../typography';
import { useSwitchContext } from '../switch-context';

export type SwitchLabelProps = TextProps;

export function SwitchLabel({ children, ...props }: SwitchLabelProps) {
  const { labelId, disabled } = useSwitchContext();

  return (
    <Text id={labelId} color={disabled ? 'whiteAlpha40' : 'white'} {...props}>
      {children}
    </Text>
  );
}
