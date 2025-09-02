import React from 'react';

import { ListboxOptionProps } from '../../lib/components/listbox/components';
import { Listbox } from '../../lib/components/listbox/Listbox';
import {
  InputListboxOptionInput,
  InputListboxOptionLabel,
  InputListboxOptionProvider,
  InputListboxOptionTrigger,
} from './components';

export type InputListboxOptionProps<T> = ListboxOptionProps<T>;

function InputListboxOption<T>({ children, ...props }: InputListboxOptionProps<T>) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const labelId = React.useId();
  return (
    <InputListboxOptionProvider inputRef={inputRef} labelId={labelId}>
      <Listbox.Option level={1} {...props}>
        <InputListboxOptionTrigger>
          <Listbox.Option.Item>
            <Listbox.Option.Content>{children}</Listbox.Option.Content>
          </Listbox.Option.Item>
        </InputListboxOptionTrigger>
      </Listbox.Option>
    </InputListboxOptionProvider>
  );
}

const InputListboxOptionNamespace = Object.assign(InputListboxOption, {
  Label: InputListboxOptionLabel,
  Input: InputListboxOptionInput,
});

export { InputListboxOptionNamespace as InputListboxOption };
