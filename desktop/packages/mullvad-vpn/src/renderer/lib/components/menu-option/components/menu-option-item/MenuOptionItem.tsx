import styled, { css } from 'styled-components';

import { colors, spacings } from '../../../../foundations';
import { MenuOptionItemIcon, MenuOptionItemLabel } from './components';

export const StyledMenuOptionItem = styled.div<{ $disabled?: boolean }>`
  ${({ $disabled }) => {
    return css`
      display: flex;
      align-items: center;
      gap: ${spacings.tiny};
      width: 100%;

      background-color: ${colors.blue40};
      padding: ${spacings.tiny} ${spacings.small};

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

const MenuOptionItem = (props: MenuOptionItemProps) => {
  return <StyledMenuOptionItem {...props} />;
};

const MenuOptionItemNamespace = Object.assign(MenuOptionItem, {
  Icon: MenuOptionItemIcon,
  Label: MenuOptionItemLabel,
});

export { MenuOptionItemNamespace as MenuOptionItem };
