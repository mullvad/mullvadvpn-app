import styled, { css } from 'styled-components';

import { spacings } from '../../../../foundations';
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

export const StyledFlexRow = styled(FlexRow)<{ $position: LocationSelectorRowPropsPositions }>`
  ${({ $position }) => {
    return css`
      position: relative;
      height: 100%;
      padding-bottom: ${$position === 'top' ? spacings.tiny : 0};
      padding-top: ${$position === 'bottom' ? spacings.tiny : 0};
    `;
  }}
`;

export const StyledLocationSelectorRowLabel = styled(BodySmall)`
  margin-left: ${spacings.big};
`;

export const StyledLocationSelectorRow = styled(Expandable)``;

function LocationSelectorRow({ position, children }: LocationSelectorRowProps) {
  const { expanded } = useLocationSelectorContext();

  return (
    <LocationSelectorRowProvider position={position}>
      <StyledLocationSelectorRow expanded={expanded}>
        <Expandable.Content>
          <StyledFlexRow
            gap="small"
            alignItems="center"
            padding={{ left: 'small' }}
            $position={position}>
            {children}
          </StyledFlexRow>
        </Expandable.Content>
      </StyledLocationSelectorRow>
    </LocationSelectorRowProvider>
  );
}

const LocationSelectorRowNamespace = Object.assign(LocationSelectorRow, {
  Icon: LocationSelectorRowIcon,
  Label: StyledLocationSelectorRowLabel,
});

export { LocationSelectorRowNamespace as LocationSelectorRow };
