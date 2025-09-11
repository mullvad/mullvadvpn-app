import React from 'react';

import { ListItem } from '../../../../../../lib/components/list-item';
import { ListItemTextFieldInputProps } from '../../../../../../lib/components/list-item/components/list-item-text-field/list-item-text-field-input';
import { useListboxContext } from '../../../../../../lib/components/listbox/components';
import { useTextField } from '../../../../../../lib/components/text-field';
import { useInputOption } from '../input-option-context';

type InputOptionInputProps = {
  initialValue?: string;
  validate?: (value: string) => boolean;
  format?: (value: string) => string;
} & ListItemTextFieldInputProps;

export function InputOptionInput({
  initialValue,
  validate,
  format,
  ...props
}: InputOptionInputProps) {
  const { onValueChange: listBoxOnValueChange, value: listBoxValue } = useListboxContext<
    string | undefined
  >();

  const { inputRef, labelId } = useInputOption();

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
        inputMode="numeric"
        onBlur={handleBlur}
        onChange={handleChange}
        {...props}
      />
    </ListItem.TextField>
  );
}
