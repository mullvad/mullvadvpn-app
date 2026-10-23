import { AnimatePresence, motion } from 'motion/react';
import styled from 'styled-components';

import { durationTokens, spacings } from '../../../../../../foundations';
import { IconButton, type IconButtonProps } from '../../../../../icon-button';

export type LocationSelectorTrailingButtonProps = IconButtonProps & {
  visible?: boolean;
};

export const StyledIconButtonWrapper = styled(motion.div)`
  display: flex;
  align-items: center;
  margin: 0 ${spacings.tiny};
`;

function LocationSelectorTrailingButton({
  visible,
  ...props
}: LocationSelectorTrailingButtonProps) {
  return visible ? (
    <AnimatePresence mode="popLayout">
      <StyledIconButtonWrapper
        layout
        animate={{ opacity: 1 }}
        initial={{ opacity: 0 }}
        exit={{ opacity: 0 }}
        transition={{ duration: durationTokens.faster, ease: 'linear' }}>
        <IconButton {...props} />
      </StyledIconButtonWrapper>
    </AnimatePresence>
  ) : null;
}
const LocationSelectorTrailingButtonNamespace = Object.assign(LocationSelectorTrailingButton, {
  Icon: IconButton.Icon,
});

export { LocationSelectorTrailingButtonNamespace as LocationSelectorTrailingButton };
