import { Text } from '../../../../lib/components';
import type { CheckboxProps } from '../../../../lib/components/checkbox';
import { Listbox } from '../../../../lib/components/listbox';
import type { ListboxOptionProps } from '../../../../lib/components/listbox/components/listbox-options/components';

export type CheckboxOptionProps<T> = ListboxOptionProps<T> & Pick<CheckboxProps, 'checked'>;

export function CheckboxOption<T>({
  value,
  animation,
  disabled,
  checked,
  children,
  ...props
}: CheckboxOptionProps<T>) {
  return (
    <Listbox.Options.Option value={value} animation={animation} disabled={disabled} {...props}>
      <Listbox.Options.Option.Trigger role="checkbox" aria-checked={checked}>
        <Listbox.Options.Option.Item>
          <Listbox.Options.Option.Item.Group gap="small">
            <Listbox.Options.Option.Item.Checkbox checked={checked}>
              <Listbox.Options.Option.Item.Checkbox.Input tabIndex={-1} />
            </Listbox.Options.Option.Item.Checkbox>
            <Text variant="bodySmallSemibold">{children}</Text>
          </Listbox.Options.Option.Item.Group>
        </Listbox.Options.Option.Item>
      </Listbox.Options.Option.Trigger>
    </Listbox.Options.Option>
  );
}
