import { Flex } from '../../lib/components';
import { ListboxOptionProps } from '../../lib/components/listbox/components';
import { Listbox } from '../../lib/components/listbox/Listbox';
import { SplitListboxOptionItem, SplitListboxOptionNavigateButton } from './components';

export type ListBoxOptionWithNavigationProps<T> = ListboxOptionProps<T>;

function SplitListboxOption<T>({ children, ...props }: ListBoxOptionWithNavigationProps<T>) {
  return (
    <Listbox.Option level={1} {...props}>
      <Flex>{children}</Flex>
    </Listbox.Option>
  );
}

const SplitListboxOptionNamespace = Object.assign(SplitListboxOption, {
  Item: SplitListboxOptionItem,
  NavigateButton: SplitListboxOptionNavigateButton,
});

export { SplitListboxOptionNamespace as SplitListboxOption };
