import styled from 'styled-components';

import { Flex } from '../../../../../flex';

export type WizardButtonGroupProps = React.ComponentPropsWithRef<'div'>;

export const StyledWizardButtonGroup = styled(Flex)``;

export function WizardButtonGroup({ children, ...props }: WizardButtonGroupProps) {
  return (
    <StyledWizardButtonGroup gap="small" {...props}>
      {children}
    </StyledWizardButtonGroup>
  );
}
