import styled from 'styled-components';

import { DeprecatedColors } from '../../../foundations';
import { Icon, IconProps } from '../../icon/Icon';
import { useIconButtonContext } from '../IconButtonContext';
export type IconButtonIconProps = IconProps;

const variants = {
  primary: {
    background: DeprecatedColors.white,
    hover: DeprecatedColors.white60,
    disabled: DeprecatedColors.white50,
  },
  secondary: {
    background: DeprecatedColors.white60,
    hover: DeprecatedColors.white80,
    disabled: DeprecatedColors.white50,
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
