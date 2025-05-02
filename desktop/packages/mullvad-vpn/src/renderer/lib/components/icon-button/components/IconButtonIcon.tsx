import styled from 'styled-components';

import { Colors, colors } from '../../../foundations';
import { Icon, IconProps } from '../../icon/Icon';
import { useIconButtonContext } from '../IconButtonContext';
export type IconButtonIconProps = IconProps;

const variants = {
  primary: {
    background: 'white',
    hover: 'whiteAlpha60',
    disabled: 'whiteAlpha40',
  },
  secondary: {
    background: 'whiteAlpha60',
    hover: 'whiteAlpha80',
    disabled: 'whiteAlpha40',
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
