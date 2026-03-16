import { Listbox } from '../../../../../../lib/components/listbox';

export type ListBoxOptionWithNavigationProps = React.ComponentPropsWithRef<'li'>;

export function SplitOptionItem({ children, ...props }: ListBoxOptionWithNavigationProps) {
  return (
    <Listbox.Options.Option.Trigger data-option data-split-button {...props}>
      <Listbox.Options.Option.Item>{children}</Listbox.Options.Option.Item>
    </Listbox.Options.Option.Trigger>
  );
}
