import styled, { css } from 'styled-components';

import { Flex, FlexProps } from '../../../flex';

const sizes = {
  full: '100%',
  small: '56px',
};

type Size = keyof typeof sizes;

const StyledFlex = styled(Flex)<{
  $size: Size;
}>`
  ${({ $size }) => {
    const size = sizes[$size];
    return css`
      --size: ${size};
      width: var(--size);
      height: 100%;
      &&:has(> :last-child:nth-child(1)) {
        &&:has(img) {
          justify-content: center;
        }
      }
    `;
  }}
`;

export interface ListItemContentProps extends FlexProps {
  size?: Size;
}

export function ListItemContent({ size = 'full', ...props }: ListItemContentProps) {
  return (
    <StyledFlex
      $size={size}
      $alignItems="center"
      $justifyContent="space-between"
      $gap="small"
      $padding={{
        horizontal: 'medium',
      }}
      {...props}
    />
  );
}
