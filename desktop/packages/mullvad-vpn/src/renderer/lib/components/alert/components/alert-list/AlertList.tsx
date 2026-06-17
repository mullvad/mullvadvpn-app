import styled from 'styled-components';

import { colors } from '../../../../foundations';

export type AlertListProps = React.ComponentPropsWithRef<'ul'>;

export const StyledAlertList = styled.ul`
  list-style: disc outside;
  padding-left: 20px;
  color: ${colors.whiteAlpha60};
`;

export function AlertList({ children, ...props }: AlertListProps) {
  return <StyledAlertList {...props}>{children}</StyledAlertList>;
}
