import React from 'react';

import { ListItem } from '../../../../../../lib/components/list-item';
import { ListItemTextFieldInputProps } from '../../../../../../lib/components/list-item/components/list-item-text-field/components';
import { useListboxContext } from '../../../../../../lib/components/listbox';
import { useInputOptionContext } from '../../InputOptionContext';

type InputOptionInputProps = ListItemTextFieldInputProps;

export function InputOptionInput(props: InputOptionInputProps) {
  const { onValueChange: listBoxOnValueChange } = useListboxContext<string | undefined>();

  const { inputRef, triggerRef, labelId, inputState } = useInputOptionContext();
  const { value, invalid, dirty, blur, handleChange, reset } = inputState;

  // Prevent the click from propagating to the ListboxOption, which would select the option.
  const handleClick = React.useCallback((event: React.MouseEvent) => {
    event.stopPropagation();
  }, []);

  const handleBlur = React.useCallback(
    (event: React.FocusEvent) => {
      const trigger = triggerRef.current;
      const next = event.relatedTarget;

      if (next instanceof Node && trigger?.contains(next)) {
        // Focus moved to the trigger, do not reset the input.
        return;
      }

      reset();
    },
    [reset, triggerRef],
  );

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
    <ListItem.TextField invalid={invalid && dirty} onSubmit={handleSubmit}>
      <ListItem.TextField.Input
        ref={inputRef}
        value={value}
        aria-labelledby={labelId}
        tabIndex={-1}
        inputMode="numeric"
        onClick={handleClick}
        onBlur={handleBlur}
        onChange={handleChange}
        {...props}
      />
    </ListItem.TextField>
  );
}
