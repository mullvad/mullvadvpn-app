import React from 'react';

import { TextField, useTextFieldContext } from '../../../../../../../../../text-field';
import type { TextFieldIconButtonProps } from '../../../../../../../../../text-field/components';

export type LocationSelectorClearButtonProps = TextFieldIconButtonProps;

export function LocationSelectorClearButton(props: LocationSelectorClearButtonProps) {
  const { onValueChange } = useTextFieldContext();

  const handleClick = React.useCallback(() => {
    onValueChange?.('');
  }, [onValueChange]);

  return (
    <TextField.IconButton onClick={handleClick} {...props}>
      <TextField.IconButton.Icon icon="cross" />
    </TextField.IconButton>
  );
}
