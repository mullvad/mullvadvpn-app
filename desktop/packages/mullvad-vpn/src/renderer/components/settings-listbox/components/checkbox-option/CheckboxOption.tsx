import { Text } from '../../../../lib/components';
import { CheckboxProps } from '../../../../lib/components/checkbox';
import { Listbox } from '../../../../lib/components/listbox';
import { ListboxOptionProps } from '../../../../lib/components/listbox/components';

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
    <Listbox.Option value={value} animation={animation} disabled={disabled} {...props}>
      <Listbox.Option.Trigger role="checkbox" aria-checked={checked}>
        <Listbox.Option.Item>
          <Listbox.Option.Group gap="small">
            <Listbox.Option.Checkbox checked={checked}>
              <Listbox.Option.Checkbox.Input tabIndex={-1} />
            </Listbox.Option.Checkbox>
            <Text variant="bodySmallSemibold">{children}</Text>
          </Listbox.Option.Group>
        </Listbox.Option.Item>
      </Listbox.Option.Trigger>
    </Listbox.Option>
  );
}
