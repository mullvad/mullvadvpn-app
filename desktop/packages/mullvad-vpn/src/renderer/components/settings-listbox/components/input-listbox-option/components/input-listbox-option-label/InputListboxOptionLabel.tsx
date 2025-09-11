import { Listbox } from '../../../../../../lib/components/listbox/Listbox';
import { useInputListboxOption } from '../input-listbox-option-context';

export type InputListboxOptionLabelProps = {
  children: React.ReactNode;
};

export function InputListboxOptionLabel({ children }: InputListboxOptionLabelProps) {
  const { labelId } = useInputListboxOption();
  return (
    <Listbox.Option.Group>
      <Listbox.Option.Icon icon="checkmark" aria-hidden="true" />
      <Listbox.Option.Label id={labelId}>{children}</Listbox.Option.Label>
    </Listbox.Option.Group>
  );
}
