import { Listbox } from '../../../../../../lib/components/listbox';
import { useInputOptionContext } from '../../InputOptionContext';

export type InputOptionLabelProps = {
  children: React.ReactNode;
};

export function InputOptionLabel({ children }: InputOptionLabelProps) {
  const { labelId } = useInputOptionContext();
  return <Listbox.Option.Label id={labelId}>{children}</Listbox.Option.Label>;
}
