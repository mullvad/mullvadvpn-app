import styled from 'styled-components';

import { Icon, type IconProps } from '../../../../../../../icon';

export type WizardSlideIconProps = IconProps;

export const StyledWizardSlideIcon = styled(Icon)`
  align-self: center;
`;

export function WizardSlideIcon(props: WizardSlideIconProps) {
  return <StyledWizardSlideIcon size="big" {...props}></StyledWizardSlideIcon>;
}
