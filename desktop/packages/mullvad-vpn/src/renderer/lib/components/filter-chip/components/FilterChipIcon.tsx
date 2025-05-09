import styled from 'styled-components';

import { Icon, IconProps } from '../../icon';
import { useFilterChipContext } from '../FilterChipContext';

type FilterChipIconProps = Omit<IconProps, 'size'>;

export const StyledIcon = styled(Icon)({});

export const FilterChipIcon = ({ ...props }: FilterChipIconProps) => {
  const { disabled } = useFilterChipContext();
  return <Icon size="small" color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
};
