import styled, { css } from 'styled-components';

import { spacings } from '../../../../foundations';
import { Flex, FlexProps } from '../../../flex';
import { useIndent } from './hooks';

const sizes = {
  full: '100%',
  small: '56px',
};

type Size = keyof typeof sizes;

const StyledFlex = styled(Flex)<{
  $size: Size;
  $paddingLeft: string;
}>`
  ${({ $size, $paddingLeft }) => {
    const size = sizes[$size];
    return css`
      --size: ${size};
      width: var(--size);
      height: 100%;
      padding-left: ${$paddingLeft};
      padding-right: ${spacings.medium};
      padding-top: ${spacings.tiny};
      padding-bottom: ${spacings.tiny};
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
  const leftPadding = useIndent();

  return (
    <StyledFlex
      alignItems="center"
      justifyContent="space-between"
      gap="small"
      $size={size}
      $paddingLeft={leftPadding}
      {...props}
    />
  );
}
