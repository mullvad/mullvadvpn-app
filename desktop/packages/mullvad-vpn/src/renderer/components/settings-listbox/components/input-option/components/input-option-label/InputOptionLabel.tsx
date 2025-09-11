import { Listbox } from '../../../../../../lib/components/listbox/Listbox';
import { useInputOptionContext } from '../../InputOptionContext';

export type InputOptionLabelProps = {
  children: React.ReactNode;
};

export function InputOptionLabel({ children }: InputOptionLabelProps) {
  const { labelId } = useInputOptionContext();
  return (
    <Listbox.Option.Group>
      <Listbox.Option.Icon icon="checkmark" />
      <Listbox.Option.Label id={labelId}>{children}</Listbox.Option.Label>
    </Listbox.Option.Group>
  );
}
