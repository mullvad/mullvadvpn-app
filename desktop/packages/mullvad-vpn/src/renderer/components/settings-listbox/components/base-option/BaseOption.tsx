import { Listbox } from '../../../../lib/components/listbox';
import type { ListboxOptionProps } from '../../../../lib/components/listbox/components/listbox-options/components';

export type BaseOptionProps<T> = ListboxOptionProps<T>;

export function BaseOption<T>({
  value,
  animation,
  disabled,
  children,
  ...props
}: BaseOptionProps<T>) {
  return (
    <Listbox.Options.Option value={value} animation={animation} disabled={disabled} {...props}>
      <Listbox.Options.Option.Trigger>
        <Listbox.Options.Option.Item>
          <Listbox.Options.Option.Item.Label>{children}</Listbox.Options.Option.Item.Label>
        </Listbox.Options.Option.Item>
      </Listbox.Options.Option.Trigger>
    </Listbox.Options.Option>
  );
}
