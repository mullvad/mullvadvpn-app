import React from 'react';

import { TextField, useTextFieldContext } from '../../../../../../../../../text-field';
import type { TextFieldIconButtonProps } from '../../../../../../../../../text-field/components/text-field-input-group/components/text-field-icon-button';

export type LocationSelectorClearButtonProps = TextFieldIconButtonProps;

export function LocationSelectorClearButton(props: LocationSelectorClearButtonProps) {
  const { onValueChange } = useTextFieldContext();

  const handleClick = React.useCallback(() => {
    onValueChange?.('');
  }, [onValueChange]);

  return (
    <TextField.InputGroup.IconButton onClick={handleClick} {...props}>
      <TextField.InputGroup.IconButton.Icon icon="cross" />
    </TextField.InputGroup.IconButton>
  );
}
