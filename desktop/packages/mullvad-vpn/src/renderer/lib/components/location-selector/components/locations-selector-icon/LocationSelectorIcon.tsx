import styled, { css } from 'styled-components';

import { spacings } from '../../../../foundations';
import { Flex } from '../../../flex';
import { Icon, type IconProps } from '../../../icon';
import type { LocationSelectorPositions } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';

export type LocationSelectorIconProps = IconProps & {
  position?: LocationSelectorPositions;
  horizontalOffset?: number;
};

export const StyledLocationSelectorIcon = styled(Flex)`
  position: absolute;
  top: 0;
  z-index: var(--location-selector-above-line-z-index);
  height: 100%;
`;

export const StyledIconContainer = styled.div<{ $horizontalOffset: number }>`
  ${({ $horizontalOffset }) => {
    return css`
      position: relative;
      top: 50%;
      z-index: inherit;
      left: ${$horizontalOffset}px;
      transform: translateY(-50%);
      height: 18px;
    `;
  }}
`;

export const StyledIcon = styled(Icon)`
  position: absolute;
  top: 50%;
  left: ${spacings.small};
  z-index: inherit;
  transform: translateY(-50%);
`;

export const StyledLine = styled(LocationSelectorLine)<{
  $position: LocationSelectorPositions;
}>`
  ${({ $position }) => {
    const top = $position === 'top' ? -10 : $position === 'bottom' ? 20 : 0;

    return css`
      left: 16.5px;
      z-index: var(--location-selector-z-index);
      top: ${top}px;
      height: 8px;
    `;
  }}
`;

export function LocationSelectorIcon({
  position = 'middle',
  horizontalOffset = 0,
  ...props
}: LocationSelectorIconProps) {
  const { expanded } = useLocationSelectorContext();

  return (
    <StyledLocationSelectorIcon aria-hidden>
      <StyledIconContainer $horizontalOffset={horizontalOffset}>
        {(position === 'bottom' || position === 'middle') && (
          <StyledLine $position="top" $visible={expanded} />
        )}
        <StyledIcon size="small" {...props} />
        {(position === 'top' || position === 'middle') && (
          <StyledLine $position="bottom" $visible={expanded} />
        )}
      </StyledIconContainer>
    </StyledLocationSelectorIcon>
  );
}
