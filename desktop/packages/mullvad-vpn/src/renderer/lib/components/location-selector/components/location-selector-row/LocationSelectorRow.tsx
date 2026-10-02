import { motion } from 'motion/react';
import styled, { css } from 'styled-components';

import { colors, spacings } from '../../../../foundations';
import { Expandable } from '../../../expandable';
import { FlexRow } from '../../../flex-row';
import { BodySmall } from '../../../text';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorRowIcon } from './components';
import { LocationSelectorRowProvider } from './LocationSelectorRowContext';

export type LocationSelectorRowPropsPositions = 'top' | 'bottom';

export type LocationSelectorRowProps = React.PropsWithChildren<{
  position: LocationSelectorRowPropsPositions;
}>;

export const StyledLocationSelectorRowContent = styled(FlexRow).attrs({
  gap: 'small',
  alignItems: 'center',
  padding: { left: 'tiny' },
})`
  ${() => {
    return css`
      position: relative;
      min-height: 28px;
    `;
  }}
`;

export const StyledLocationSelectorRowLabel = styled(BodySmall)`
  margin-left: ${spacings.big};
  color: ${colors.whiteAlpha60};
`;

function LocationSelectorRow({ position, children }: LocationSelectorRowProps) {
  const { expanded } = useLocationSelectorContext();

  return (
    <LocationSelectorRowProvider position={position}>
      <motion.div layout="position" transition={{ duration: 0.25, ease: 'easeOut' }}>
        <Expandable expanded={expanded} initial={false}>
          <Expandable.Content transition={{ duration: 0.25, ease: 'easeOut' }}>
            {children}
          </Expandable.Content>
        </Expandable>
      </motion.div>
    </LocationSelectorRowProvider>
  );
}

const LocationSelectorRowNamespace = Object.assign(LocationSelectorRow, {
  Icon: LocationSelectorRowIcon,
  Label: StyledLocationSelectorRowLabel,
  Content: StyledLocationSelectorRowContent,
});

export { LocationSelectorRowNamespace as LocationSelectorRow };
