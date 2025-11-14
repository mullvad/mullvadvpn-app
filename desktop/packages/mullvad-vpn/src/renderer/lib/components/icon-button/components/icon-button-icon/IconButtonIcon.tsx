import styled from 'styled-components';

import { Icon, IconProps } from '../../../icon/Icon';
import { useIconButtonContext } from '../../IconButtonContext';
export type IconButtonIconProps = IconProps;

export const StyledIconButtonIcon = styled(Icon)``;

export const IconButtonIcon = (props: IconButtonIconProps) => {
  const { size } = useIconButtonContext();
  return <StyledIconButtonIcon size={size} aria-hidden {...props} />;
};
