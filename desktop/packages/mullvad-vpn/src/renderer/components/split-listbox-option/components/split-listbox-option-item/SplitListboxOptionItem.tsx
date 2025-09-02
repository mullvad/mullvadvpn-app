import { Listbox } from '../../../../lib/components/listbox/Listbox';

export type ListBoxOptionWithNavigationProps = React.ComponentPropsWithRef<'li'>;

export function SplitListboxOptionItem({ children, ...props }: ListBoxOptionWithNavigationProps) {
  return (
    <Listbox.Option.Trigger {...props}>
      <Listbox.Option.Item>
        <Listbox.Option.Content>
          <Listbox.Option.Group>
            <Listbox.Option.Icon icon="checkmark" />
            {children}
          </Listbox.Option.Group>
        </Listbox.Option.Content>
      </Listbox.Option.Item>
    </Listbox.Option.Trigger>
  );
}
