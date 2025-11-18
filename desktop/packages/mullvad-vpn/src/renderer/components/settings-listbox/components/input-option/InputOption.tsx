import React from 'react';

import { Listbox } from '../../../../lib/components/listbox';
import { ListboxOptionProps } from '../../../../lib/components/listbox/components';
import { useTextField } from '../../../../lib/components/text-field';
import { InputOptionInput, InputOptionLabel, InputOptionTrigger } from './components';
import { InputOptionProvider } from './InputOptionContext';

export type InputOptionProps<T> = ListboxOptionProps<T> & {
  defaultValue?: string;
  validate?: (value: string) => boolean;
  format?: (value: string) => string;
};

function InputOption<T>({
  defaultValue,
  validate,
  format,
  children,
  ...props
}: InputOptionProps<T>) {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const triggerRef = React.useRef<HTMLLIElement>(null);
  const labelId = React.useId();
  const inputState = useTextField({
    inputRef,
    defaultValue,
    validate,
    format,
  });

  return (
    <InputOptionProvider
      inputRef={inputRef}
      triggerRef={triggerRef}
      labelId={labelId}
      inputState={inputState}>
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
