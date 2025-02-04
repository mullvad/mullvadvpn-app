import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { Icon, IconProps } from '../../icon/Icon';
import { useIconButtonContext } from '../IconButtonContext';
export type IconButtonIconProps = IconProps;

const variants = {
  primary: {
    background: Colors.white,
    hover: Colors.white60,
    disabled: Colors.white50,
  },
  secondary: {
    background: Colors.white60,
    hover: Colors.white80,
    disabled: Colors.white50,
  },
} as const;

const StyledIcon = styled(Icon)<IconButtonIconProps & { $hoverColor: string; $disabled?: boolean }>(
  ({ $hoverColor, $disabled }) => ({
    ...(!$disabled && {
      '&&:hover': {
        backgroundColor: $hoverColor,
      },
    }),
  }),
);

export const IconButtonIcon = (props: IconButtonIconProps) => {
  const { variant = 'primary', size, disabled } = useIconButtonContext();
  const styles = variants[variant];
  return (
    <StyledIcon
      color={disabled ? styles.disabled : styles.background}
      size={size}
      $hoverColor={styles.hover}
      $disabled={disabled}
      {...props}
    />
  );
};
