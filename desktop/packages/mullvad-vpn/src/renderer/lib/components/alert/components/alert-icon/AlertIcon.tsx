import styled from 'styled-components';

import { Icon, IconProps } from '../../../icon';

export type AlertIconProps = IconProps;

const StyledAlertIcon = styled(Icon)`
  align-self: center;
`;

export function AlertIcon(props: AlertIconProps) {
  return <StyledAlertIcon size="big" aria-hidden="true" {...props} />;
}
