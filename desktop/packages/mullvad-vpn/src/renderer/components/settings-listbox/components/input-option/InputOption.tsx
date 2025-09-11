import React from 'react';

import { ListboxOptionProps } from '../../../../lib/components/listbox/components';
import { Listbox } from '../../../../lib/components/listbox/Listbox';
import { InputOptionInput, InputOptionLabel, InputOptionTrigger } from './components';
import { InputOptionProvider } from './InputOptionContext';

export type InputOptionProps<T> = ListboxOptionProps<T>;

function InputOption<T>({ children, ...props }: InputOptionProps<T>) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const labelId = React.useId();
  return (
    <InputOptionProvider inputRef={inputRef} labelId={labelId}>
      <Listbox.Option level={1} {...props}>
        <InputOptionTrigger>
          <Listbox.Option.Item>
            <Listbox.Option.Content>{children}</Listbox.Option.Content>
          </Listbox.Option.Item>
        </InputOptionTrigger>
      </Listbox.Option>
    </InputOptionProvider>
  );
}

const InputOptionNamespace = Object.assign(InputOption, {
  Label: InputOptionLabel,
  Input: InputOptionInput,
});

export { InputOptionNamespace as InputOption };
