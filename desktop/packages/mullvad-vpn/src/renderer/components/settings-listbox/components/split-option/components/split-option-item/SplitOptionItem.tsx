import { Listbox } from '../../../../../../lib/components/listbox';

export type ListBoxOptionWithNavigationProps = React.ComponentPropsWithRef<'li'>;

export function SplitOptionItem({ children, ...props }: ListBoxOptionWithNavigationProps) {
  return (
    <Listbox.Option.Trigger data-option data-split-button {...props}>
      <Listbox.Option.Item>
        <Listbox.Option.Content>
          <Listbox.Option.Group>{children}</Listbox.Option.Group>
        </Listbox.Option.Content>
      </Listbox.Option.Item>
    </Listbox.Option.Trigger>
  );
}
