import React from 'react';

import { TextField, TextFieldProps } from '../../lib/components/text-field';
import { SearchTextFieldClearButton } from './components';

export type SearchTextFieldProps = TextFieldProps;

function SearchTextField({ value, onValueChange, ...props }: SearchTextFieldProps) {
  const [internalValue, setInternalValue] = React.useState(value);

  const deferredValue = React.useDeferredValue(internalValue);

  React.useEffect(() => {
    if (deferredValue === undefined) {
      return;
    }

    onValueChange?.(deferredValue);
  }, [deferredValue, onValueChange]);

  return <TextField value={internalValue} onValueChange={setInternalValue} {...props} />;
}

const SearchTextFieldNamespace = Object.assign(SearchTextField, {
  Input: TextField.Input,
  Label: TextField.Label,
  Icon: TextField.Icon,
  ClearButton: SearchTextFieldClearButton,
});

export { SearchTextFieldNamespace as SearchTextField };
