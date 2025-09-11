import { Listbox } from '../../../../../../lib/components/listbox';
import { useInputOption } from '../input-option-context';

export type InputOptionLabelProps = {
  children: React.ReactNode;
};

export function InputOptionLabel({ children }: InputOptionLabelProps) {
  const { labelId } = useInputOption();
  return (
    <Listbox.Option.Group>
      <Listbox.Option.Icon icon="checkmark" aria-hidden="true" />
      <Listbox.Option.Label id={labelId}>{children}</Listbox.Option.Label>
    </Listbox.Option.Group>
  );
}
