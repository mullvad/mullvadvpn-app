import styled from 'styled-components';

import { Icon, IconProps } from '../../icon';

type ButtonIconProps = Omit<IconProps, 'size'>;

export const StyledIcon = styled(Icon)({});

export const ButtonIcon = ({ ...props }: ButtonIconProps) => {
  return <StyledIcon size="medium" {...props} />;
};
