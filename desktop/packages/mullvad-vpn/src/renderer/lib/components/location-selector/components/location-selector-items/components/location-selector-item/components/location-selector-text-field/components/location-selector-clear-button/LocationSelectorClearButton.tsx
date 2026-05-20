import React from 'react';

import { TextField, useTextFieldContext } from '../../../../../../../../../text-field';
import type { TextFieldIconButtonProps } from '../../../../../../../../../text-field/components';
import { useLocationSelectorItemContext } from '../../../../LocationSelectorItemContext';

export type LocationSelectorClearButtonProps = TextFieldIconButtonProps;

export function LocationSelectorClearButton(props: LocationSelectorClearButtonProps) {
  const { value, onValueChange } = useTextFieldContext();
  const { focusInsideTextField } = useLocationSelectorItemContext();

  const handleClick = React.useCallback(() => {
    onValueChange?.('');
  }, [onValueChange]);

  const visible = focusInsideTextField && !!value;

  return visible ? (
    <TextField.IconButton onClick={handleClick} {...props}>
      <TextField.IconButton.Icon icon="cross" />
    </TextField.IconButton>
  ) : null;
}
