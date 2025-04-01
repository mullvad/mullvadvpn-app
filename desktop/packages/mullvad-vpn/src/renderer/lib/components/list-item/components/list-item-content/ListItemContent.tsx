import styled from 'styled-components';

import { Flex, FlexProps } from '../../../flex';
import { levels } from '../../levels';
import { useListItem } from '../../ListItemContext';

const sizes = {
  full: '100%',
  small: '44px',
};

export const StyledFlex = styled(Flex)<{
  $level: keyof typeof levels;
  $disabled?: boolean;
  $size: 'full' | 'small';
}>`
  width: ${({ $size }) => sizes[$size]};
  height: 100%;
  background-color: ${({ $disabled, $level }) =>
    $disabled ? levels[$level].disabled : levels[$level].enabled};
  &&:has(> :last-child:nth-child(1)) {
    &&:has(img) {
      justify-content: center;
    }
  }
`;

export interface ListItemContainerProps extends FlexProps {
  size?: 'full' | 'small';
}

export const ListItemContent = ({ size = 'full', ...props }: ListItemContainerProps) => {
  const { level } = useListItem();
  return (
    <StyledFlex
      $size={size}
      $level={level}
      $alignItems="center"
      $justifyContent="space-between"
      $gap="small"
      $padding={{
        horizontal: 'medium',
      }}
      {...props}
    />
  );
};
