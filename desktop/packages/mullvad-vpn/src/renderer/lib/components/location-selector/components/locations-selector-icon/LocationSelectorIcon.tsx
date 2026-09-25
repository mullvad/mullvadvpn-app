import styled, { css } from 'styled-components';

import { type Colors, colors, spacings } from '../../../../foundations';
import { Flex } from '../../../flex';
import { Icon, type IconProps } from '../../../icon';
import type { LocationSelectorPositions } from '../../LocationSelector';
import { useLocationSelectorContext } from '../../LocationSelectorContext';
import { LocationSelectorLine } from '../location-selector-line';

export type LocationSelectorIconProps = IconProps & {
  backgroundColor?: Colors;
  position?: LocationSelectorPositions;
  horizontalOffset?: number;
};

export const StyledLocationSelectorIcon = styled(Flex)`
  position: absolute;
  height: 100%;
  top: 0;
`;

export const StyledIconContainer = styled.div<{ $horizontalOffset: number }>`
  ${({ $horizontalOffset }) => {
    return css`
      position: relative;
      display: inline-flex;
      top: 50%;
      left: ${$horizontalOffset}px;
      transform: translateY(-50%);
      z-index: var(--above-line-z-index);
    `;
  }}
`;

export const StyledIconBackground = styled.div<{ $color: string }>`
  ${({ $color }) => {
    return css`
      height: 20px;
      width: 20px;
      position: absolute;
      top: 40%;
      left: 0px;
      z-index: var(--above-line-z-index);
      background-color: ${$color};
      transform: rotate(45deg) translateY(-50%);

      // Creates illusion that lines have rounded ends by placing an element with concave corners
      // and same color as background above the line
      corner-shape: scoop;
      border-radius: 3px;
    `;
  }}
`;

export const StyledIcon = styled(Icon)`
  position: absolute;
  top: 50%;
  left: ${spacings.small};
  transform: translateY(-50%);
  z-index: var(--above-line-z-index);
`;

export const StyledLine = styled(LocationSelectorLine)<{
  $position: LocationSelectorPositions;
}>`
  ${({ $position }) => {
    const verticalOffset = $position === 'top' ? 3 : $position === 'bottom' ? -3 : 0;

    return css`
      left: 16px;
      z-index: var(--above-line-z-index);
      top: ${verticalOffset}px;
      ${() => {
        if (verticalOffset !== 0) {
          return css`
            height: calc(100% - 1px);
          `;
        }
        return null;
      }}
    `;
  }}
`;

export function LocationSelectorIcon({
  position = 'middle',
  backgroundColor: backgroundColorProp = 'white',
  horizontalOffset = 0,
  ...props
}: LocationSelectorIconProps) {
  const { expanded } = useLocationSelectorContext();
  const backgroundColor = colors[backgroundColorProp];

  return (
    <StyledLocationSelectorIcon aria-hidden>
      <StyledIconContainer $horizontalOffset={horizontalOffset}>
        <StyledLine $position={position} $visible={expanded} />
        <StyledIconBackground $color={backgroundColor} />
        <StyledIcon size="small" {...props} />
      </StyledIconContainer>
    </StyledLocationSelectorIcon>
  );
}
