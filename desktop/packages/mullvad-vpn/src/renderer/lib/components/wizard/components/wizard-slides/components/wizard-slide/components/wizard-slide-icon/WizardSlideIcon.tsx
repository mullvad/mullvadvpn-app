import styled from 'styled-components';

import { Icon, type IconProps } from '../../../../../../../icon';

export type WizardSlideIconProps = IconProps;

export const StyledSlideIcon = styled(Icon)`
  align-self: center;
`;

export function WizardSlideIcon(props: WizardSlideIconProps) {
  return <StyledSlideIcon size="big" {...props}></StyledSlideIcon>;
}
