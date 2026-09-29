import { AnimatePresence, motion } from 'motion/react';
import React from 'react';

import { TextField, useTextFieldContext } from '../../../../../../../../../text-field';
import type { TextFieldIconButtonProps } from '../../../../../../../../../text-field/components/text-field-input-group/components/text-field-icon-button';
import { useLocationSelectorItemContext } from '../../../../LocationSelectorItemContext';

export type LocationSelectorClearButtonProps = TextFieldIconButtonProps;

export function LocationSelectorClearButton(props: LocationSelectorClearButtonProps) {
  const { value, onValueChange } = useTextFieldContext();
  const { focusInsideTextField } = useLocationSelectorItemContext();

  const handleClick = React.useCallback(() => {
    onValueChange?.('');
  }, [onValueChange]);

  const visible = focusInsideTextField && !!value;

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          layout
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.1, delay: 0.2, ease: 'linear' }}>
          <TextField.InputGroup.IconButton onClick={handleClick} {...props}>
            <TextField.InputGroup.IconButton.Icon icon="cross" />
          </TextField.InputGroup.IconButton>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
