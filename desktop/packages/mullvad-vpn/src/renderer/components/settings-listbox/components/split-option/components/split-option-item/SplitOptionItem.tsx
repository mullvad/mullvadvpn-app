import { Listbox } from '../../../../../../lib/components/listbox';

export type ListBoxOptionWithNavigationProps = React.ComponentPropsWithRef<'li'>;

export function SplitOptionItem({ children, ...props }: ListBoxOptionWithNavigationProps) {
  return (
    <Listbox.Option.Trigger data-option data-split-button {...props}>
      <Listbox.Option.Item>{children}</Listbox.Option.Item>
    </Listbox.Option.Trigger>
  );
}
