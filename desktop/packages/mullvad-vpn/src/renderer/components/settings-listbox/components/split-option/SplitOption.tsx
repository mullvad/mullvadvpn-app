import { Flex } from '../../../../lib/components';
import { Listbox } from '../../../../lib/components/listbox';
import { ListboxOptionProps } from '../../../../lib/components/listbox/components';
import { SplitOptionItem, SplitOptionNavigateButton } from './components';

export type SplitOptionProps<T> = ListboxOptionProps<T>;

function SplitOption<T>({ children, ...props }: SplitOptionProps<T>) {
  return (
    <Listbox.Option level={1} {...props}>
      <Flex>{children}</Flex>
    </Listbox.Option>
  );
}

const SplitOptionNamespace = Object.assign(SplitOption, {
  Item: SplitOptionItem,
  NavigateButton: SplitOptionNavigateButton,
  Label: Listbox.Option.Label,
});

export { SplitOptionNamespace as SplitOption };
