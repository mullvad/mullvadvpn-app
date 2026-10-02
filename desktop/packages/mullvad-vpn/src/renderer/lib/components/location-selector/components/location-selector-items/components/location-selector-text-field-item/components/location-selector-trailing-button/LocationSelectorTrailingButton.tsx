import { AnimatePresence, motion } from 'motion/react';
import styled from 'styled-components';

import { spacings } from '../../../../../../../../foundations';
import { IconButton, type IconButtonProps } from '../../../../../../../icon-button';

export type LocationSelectorTrailingButtonProps = IconButtonProps & {
  visible?: boolean;
};

export const StyledLocationSelectorTrailingButton = styled(IconButton)`
  &:last-child {
    margin-right: ${spacings.tiny};
  }
`;

function LocationSelectorTrailingButton({
  visible,
  ...props
}: LocationSelectorTrailingButtonProps) {
  return visible ? (
    <AnimatePresence mode="popLayout">
      <motion.div
        layout
        animate={{ opacity: 1 }}
        initial={{ opacity: 0 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.1, ease: 'linear' }}>
        <StyledLocationSelectorTrailingButton {...props} />
      </motion.div>
    </AnimatePresence>
  ) : null;
}
const LocationSelectorTrailingButtonNamespace = Object.assign(LocationSelectorTrailingButton, {
  Icon: IconButton.Icon,
});

export { LocationSelectorTrailingButtonNamespace as LocationSelectorTrailingButton };
