import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { Icon, IconProps } from '../../icon';
import { useButtonContext } from '../ButtonContext';

type ButtonIconProps = Omit<IconProps, 'size'>;

export const StyledIcon = styled(Icon)({});

export const ButtonIcon = ({ ...props }: ButtonIconProps) => {
  const { disabled } = useButtonContext();
  return (
    <StyledIcon
      size="medium"
      aria-hidden="true"
      color={disabled ? Colors.white40 : Colors.white}
      {...props}
    />
  );
};
