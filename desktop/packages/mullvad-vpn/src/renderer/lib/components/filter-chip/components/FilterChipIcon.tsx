import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { Icon, IconProps } from '../../icon';
import { useFilterChipContext } from '../FilterChipContext';

type FilterChipIconProps = Omit<IconProps, 'size'>;

export const StyledIcon = styled(Icon)({});

export const FilterChipIcon = ({ ...props }: FilterChipIconProps) => {
  const { disabled } = useFilterChipContext();
  return <Icon size="small" color={disabled ? Colors.white40 : Colors.white} {...props} />;
};
