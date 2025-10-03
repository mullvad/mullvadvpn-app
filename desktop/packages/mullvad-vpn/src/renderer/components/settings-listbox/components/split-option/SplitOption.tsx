import React from 'react';

import { Flex } from '../../../../lib/components';
import { Listbox } from '../../../../lib/components/listbox';
import { ListboxOptionProps } from '../../../../lib/components/listbox/components';
import { useOptions } from '../../../../lib/hooks';
import { SplitOptionItem, SplitOptionNavigateButton } from './components';

export type SplitOptionProps<T> = ListboxOptionProps<T>;

function SplitOption<T>({ children, ...props }: SplitOptionProps<T>) {
  const optionsRef = React.useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = React.useState<number | undefined>(undefined);
  const { handleKeyboardNavigation, handleBlur, handleFocus } = useOptions({
    optionsRef,
    orientation: 'horizontal',
    selector: '[data-split-button="true"]',
    focusedIndex,
    setFocusedIndex,
  });

  return (
    <Listbox.Option
      level={1}
      role="group"
      onKeyDown={handleKeyboardNavigation}
      onFocus={handleFocus}
      onBlur={handleBlur}
      {...props}>
      <Flex ref={optionsRef}>{children}</Flex>
    </Listbox.Option>
  );
}

const SplitOptionNamespace = Object.assign(SplitOption, {
  Item: SplitOptionItem,
  NavigateButton: SplitOptionNavigateButton,
  Label: Listbox.Option.Label,
});

export { SplitOptionNamespace as SplitOption };
