import { Label, LabelProps } from '../../../label';
import { useCheckboxContext } from '../../CheckboxContext';

export type CheckboxLabelProps = LabelProps;

export function CheckboxLabel({ children, ...props }: CheckboxLabelProps) {
  const { inputId, disabled } = useCheckboxContext();

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
