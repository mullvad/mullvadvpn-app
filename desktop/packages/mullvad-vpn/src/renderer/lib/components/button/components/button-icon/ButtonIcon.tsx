import styled from 'styled-components';

import { Icon, IconProps } from '../../../icon';
import { useButtonContext } from '../../ButtonContext';

type ButtonIconProps = Omit<IconProps, 'size'>;

export const StyledButtonIcon = styled(Icon)({});

export function ButtonIcon({ ...props }: ButtonIconProps) {
  const { disabled } = useButtonContext();
  return (
    <StyledButtonIcon
      size="medium"
      aria-hidden="true"
      color={disabled ? 'whiteAlpha40' : 'white'}
      {...props}
    />
  );
}
