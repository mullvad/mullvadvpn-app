import styled from 'styled-components';

import { Colors, colors } from '../../../foundations';
import { Icon, IconProps } from '../../icon/Icon';
import { useIconButtonContext } from '../IconButtonContext';
export type IconButtonIconProps = IconProps;

const variants = {
  primary: {
    background: 'white100',
    hover: 'white60',
    disabled: 'white40',
  },
  secondary: {
    background: 'white60',
    hover: 'white80',
    disabled: 'white40',
  },
} as const;

const StyledIcon = styled(Icon)<IconButtonIconProps & { $hoverColor: Colors; $disabled?: boolean }>(
  ({ $hoverColor, $disabled }) => {
    const hoverColor = colors[$hoverColor];
    return {
      ...(!$disabled && {
        '&&:hover': {
          backgroundColor: hoverColor,
        },
      }),
    };
  },
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
