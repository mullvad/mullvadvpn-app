import { Listbox } from '../../../../lib/components/listbox';
import { ListboxOptionProps } from '../../../../lib/components/listbox/components';

export type BaseOptionProps<T> = ListboxOptionProps<T>;

export function BaseOption<T>({
  value,
  animation,
  disabled,
  children,
  ...props
}: BaseOptionProps<T>) {
  return (
    <Listbox.Option value={value} animation={animation} disabled={disabled} {...props}>
      <Listbox.Option.Trigger>
        <Listbox.Option.Item>
          <Listbox.Option.Content>
            <Listbox.Option.Group>
              <Listbox.Option.Label>{children}</Listbox.Option.Label>
            </Listbox.Option.Group>
          </Listbox.Option.Content>
        </Listbox.Option.Item>
      </Listbox.Option.Trigger>
    </Listbox.Option>
  );
}
