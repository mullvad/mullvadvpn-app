import React from 'react';

import { ListItem } from '../../../../lib/components/list-item';
import { ListItemTextFieldInputProps } from '../../../../lib/components/list-item/components/list-item-text-field/list-item-text-field-input';
import { useListboxContext } from '../../../../lib/components/listbox/components';
import { useTextField } from '../../../../lib/components/text-field';
import { useInputListboxOption } from '../input-listbox-option-context';

type InputListboxOptionInputProps = {
  initialValue?: string;
  validate?: (value: string) => boolean;
  format?: (value: string) => string;
} & ListItemTextFieldInputProps;

export function InputListboxOptionInput({
  initialValue,
  validate,
  format,
  ...props
}: InputListboxOptionInputProps) {
  const { onValueChange: listBoxOnValueChange, value: listBoxValue } = useListboxContext<
    string | undefined
  >();

  const { inputRef, labelId } = useInputListboxOption();

  const { value, invalid, dirty, blur, handleChange, reset } = useTextField({
    inputRef,
    defaultValue: initialValue,
    validate,
    format,
  });

  React.useEffect(() => {
    if (listBoxValue !== 'custom') {
      reset();
    }
  }, [listBoxValue, reset]);

  const handleBlur = React.useCallback(async () => {
    if (listBoxOnValueChange && !invalid && dirty) {
      await listBoxOnValueChange(value);
    }
    if (invalid) {
      reset();
    }
  }, [dirty, invalid, listBoxOnValueChange, reset, value]);

  const handleSubmit = React.useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (listBoxOnValueChange && !invalid) {
        await listBoxOnValueChange?.(value);
        blur();
      }
    },
    [blur, invalid, listBoxOnValueChange, value],
  );

  return (
    <ListItem.TextField invalid={invalid} onSubmit={handleSubmit}>
      <ListItem.TextField.Input
        ref={inputRef}
        value={value}
        aria-labelledby={labelId}
        tabIndex={-1}
        inputMode="numeric"
        onBlur={handleBlur}
        onChange={handleChange}
        {...props}
      />
    </ListItem.TextField>
  );
}
