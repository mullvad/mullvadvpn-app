import React from 'react';
import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../../../../../foundations';
import { BodySmall, StyledText } from '../../../../../../../text';
import { useIsLocationSelected } from '../../../../hooks';
import { useLocationSelectorButtonItemContext } from '../../LocationSelectorItemContext';
import { LocationSelectorButtonIcon } from './components';

export type LocationSelectorButtonProps = Omit<
  React.ComponentPropsWithRef<'button'>,
  'children'
> & {
  label?: string;
};

const StyledLocationSelectorButton = styled.button<{
  $selected: boolean;
}>`
  ${({ $selected }) => {
    return css`
      position: relative;
      height: 32px;
      border-radius: ${Radius.radius12};
      width: 100%;

      padding: ${spacings.tiny} ${spacings.small};
      margin: 1px calc(1px + ${spacings.tiny}) 1px calc(1px + ${spacings.tiny});

      color: ${colors.white};
      background-color: ${colors.darkerBlue10};
      outline: 1px solid ${colors.chalkAlpha40};

      &:not(:disabled):hover {
        outline-color: ${colors.chalkAlpha80};
      }
      &:not(:disabled):focus-visible {
        outline-width: 2px;
        outline-offset: -1px;
      }
      &:not(:disabled):focus-visible {
        outline-color: ${colors.chalk};
      }

      ${() => {
        if ($selected) {
          return css`
            background-color: ${colors.blue40};
          `;
        }
        return null;
      }}

      & > ${StyledText} {
        padding-left: calc(18px + ${spacings.tiny});
        align-self: center;
      }
    `;
  }}
`;

export function LocationSelectorButton({
  onClick: onClickProp,
  label,
  ...props
}: LocationSelectorButtonProps) {
  const { id, onSelectedItemChange } = useLocationSelectorButtonItemContext();

  const selected = useIsLocationSelected(id);

  const handleClick = React.useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      onSelectedItemChange?.(id);
      onClickProp?.(event);
    },
    [onClickProp, onSelectedItemChange, id],
  );

  return (
    <StyledLocationSelectorButton onClick={handleClick} $selected={selected} {...props}>
      <LocationSelectorButtonIcon
        icon="magic-multihop"
        color={selected ? 'white' : 'whiteAlpha60'}
      />
      <BodySmall>{label}</BodySmall>
    </StyledLocationSelectorButton>
  );
}
