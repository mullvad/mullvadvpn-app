import React from 'react';

import { TextField, TextFieldProps } from '../../lib/components/text-field';
import { useDebounce } from '../../lib/hooks/use-debounce';
import { createComponentNamespace } from '../../utils';
import { SearchTextFieldClearButton } from './components';

export type SearchTextFieldProps = TextFieldProps & {
  delay?: number;
};

function SearchTextField({ value, onValueChange, delay = 200, ...props }: SearchTextFieldProps) {
  const [internalValue, setInternalValue] = React.useState(value);
  const debouncedValue = useDebounce(internalValue, internalValue === '' ? 0 : delay);

  React.useEffect(() => {
    if (debouncedValue === undefined) {
      return;
    }

    onValueChange?.(debouncedValue);
  }, [debouncedValue, onValueChange]);

  return <TextField value={internalValue} onValueChange={setInternalValue} {...props} />;
}

const SearchTextFieldNamespace = createComponentNamespace(SearchTextField, {
  InputGroup: createComponentNamespace(TextField.InputGroup, {
    ClearButton: SearchTextFieldClearButton,
  }),
});

export { SearchTextFieldNamespace as SearchTextField };
