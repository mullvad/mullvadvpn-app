import styled from 'styled-components';

import { Icon, IconProps } from '../../icon';
import { useFilterChipContext } from '../FilterChipContext';

type FilterChipIconProps = Omit<IconProps, 'size'>;

export const StyledFilterChipIcon = styled(Icon)``;

export const FilterChipIcon = ({ ...props }: FilterChipIconProps) => {
  const { disabled } = useFilterChipContext();
  return (
    <StyledFilterChipIcon
      size="small"
      color={disabled ? 'whiteAlpha40' : 'whiteAlpha60'}
      {...props}
    />
  );
};
