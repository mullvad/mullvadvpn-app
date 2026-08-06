import styled from 'styled-components';

import { Icon, IconProps } from '../../../icon';
import type { ButtonProps } from '../../Button';
import { useButtonContext } from '../../ButtonContext';

type ButtonIconProps = Omit<IconProps, 'size'> & Pick<ButtonProps, 'disabled'>;

export const StyledButtonIcon = styled(Icon)``;

export function ButtonIcon({ disabled: disabledProp, ...props }: ButtonIconProps) {
  const { disabled: disabledContext } = useButtonContext();
  const disabled = disabledProp ?? disabledContext;

  return (
    <StyledButtonIcon
      size="medium"
      aria-hidden="true"
      color={disabled ? 'whiteAlpha40' : 'white'}
      {...props}
    />
  );
}
