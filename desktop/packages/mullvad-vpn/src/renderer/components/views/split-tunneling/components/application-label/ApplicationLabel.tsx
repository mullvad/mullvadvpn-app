import styled from 'styled-components';

import { Label } from '../../../../cell';
import { normalText } from '../../../../common-styles';
import { disabledApplication, type DisabledApplicationProps } from '../../utils';

export const StyledCellLabel = styled(Label)<DisabledApplicationProps>(
  disabledApplication,
  normalText,
  {
    fontWeight: 400,
    wordWrap: 'break-word',
    overflow: 'hidden',
  },
);

export type ApplicationLabelProps = {
  children: React.ReactNode;
  disabled?: boolean;
};

export function ApplicationLabel({ children, disabled }: ApplicationLabelProps) {
  return <StyledCellLabel $lookDisabled={disabled}>{children}</StyledCellLabel>;
}
