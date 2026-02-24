import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';

export const StyledMenuOptionItem = styled.div<{ $disabled?: boolean }>`
  ${({ $disabled }) => {
    return css`
      display: flex;
      flex-direction: row;
      gap: ${spacings.tiny};
      width: 100%;

      background-color: ${colors.blue40};
      padding: ${spacings.tiny} ${spacings.small};
      border-radius: ${Radius.radius4};

      ${() => {
        if ($disabled) {
          return css`
            background-color: ${colors.whiteOnBlue5};
          `;
        }
        return null;
      }}
    `;
  }}
`;

export type MenuOptionItemProps = React.ComponentPropsWithoutRef<'div'>;

export const MenuOptionItem = (props: MenuOptionItemProps) => {
  return <StyledMenuOptionItem {...props} />;
};
