import React from 'react';

import { TextField, useTextFieldContext } from '../../../../lib/components/text-field';
import { TextFieldIconButtonProps } from '../../../../lib/components/text-field/components';

export type SearchTextFieldClearButtonProps = TextFieldIconButtonProps;

export function SearchTextFieldClearButton(props: SearchTextFieldClearButtonProps) {
  const { value, onValueChange } = useTextFieldContext();

  const handleClick = React.useCallback(() => {
    onValueChange?.('');
  }, [onValueChange]);

  return value ? (
    <TextField.IconButton onClick={handleClick} {...props}>
      <TextField.IconButton.Icon icon="cross" />
    </TextField.IconButton>
  ) : null;
}
