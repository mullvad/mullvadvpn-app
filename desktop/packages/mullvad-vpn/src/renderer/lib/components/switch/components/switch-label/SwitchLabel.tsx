import { Label, LabelProps } from '../../../label';
import { useSwitchContext } from '../../';

export type SwitchLabelProps = LabelProps;

export function SwitchLabel({ children, ...props }: SwitchLabelProps) {
  const { inputId, disabled } = useSwitchContext();

  return (
    <Label
      htmlFor={inputId}
      variant="bodySmallSemibold"
      color={disabled ? 'whiteAlpha40' : 'white'}
      {...props}>
      {children}
    </Label>
  );
}
