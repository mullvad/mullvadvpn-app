import React from 'react';

import { TextField, useTextFieldContext } from '../../../../lib/components/text-field';
import { type TextFieldIconButtonProps } from '../../../../lib/components/text-field/components/text-field-input-group/components';

export type SearchTextFieldClearButtonProps = TextFieldIconButtonProps;

export function SearchTextFieldClearButton(props: SearchTextFieldClearButtonProps) {
  const { value, onValueChange } = useTextFieldContext();

  const handleClick = React.useCallback(() => {
    onValueChange?.('');
  }, [onValueChange]);

  return value ? (
    <TextField.InputGroup.IconButton onClick={handleClick} {...props}>
      <TextField.InputGroup.IconButton.Icon icon="cross" />
    </TextField.InputGroup.IconButton>
  ) : null;
}
