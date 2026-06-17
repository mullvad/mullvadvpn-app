import styled from 'styled-components';

import { IconBadge, type IconBadgeProps } from '../../../../icon-badge';

export type AlertIconBadgeProps = IconBadgeProps;

export const StyledAlertIconBadge = styled(IconBadge)`
  align-self: center;
`;

export function AlertIconBadge(props: AlertIconBadgeProps) {
  return <StyledAlertIconBadge aria-hidden="true" {...props} />;
}
