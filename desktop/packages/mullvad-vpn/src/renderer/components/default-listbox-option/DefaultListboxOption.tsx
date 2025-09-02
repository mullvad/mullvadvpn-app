import { ListboxOptionProps } from '../../lib/components/listbox/components';
import { ListboxOptionTriggerProps } from '../../lib/components/listbox/components/listbox-option/components';
import { Listbox } from '../../lib/components/listbox/Listbox';

export type DefaultListboxOptionProps<T> = ListboxOptionProps<T> &
  Omit<ListboxOptionTriggerProps, keyof ListboxOptionProps<T>>;

export function DefaultListboxOption<T>({
  value,
  animation,
  disabled,
  children,
  ...props
}: DefaultListboxOptionProps<T>) {
  return (
    <Listbox.Option level={1} value={value} animation={animation} disabled={disabled}>
      <Listbox.Option.Trigger {...props}>
        <Listbox.Option.Item>
          <Listbox.Option.Content>
            <Listbox.Option.Group>
              <Listbox.Option.Icon icon="checkmark" />
              <Listbox.Option.Label>{children}</Listbox.Option.Label>
            </Listbox.Option.Group>
          </Listbox.Option.Content>
        </Listbox.Option.Item>
      </Listbox.Option.Trigger>
    </Listbox.Option>
  );
}
