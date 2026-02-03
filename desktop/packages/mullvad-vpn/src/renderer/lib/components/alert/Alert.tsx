import styled from 'styled-components';

import { Flex } from '../flex';
import { AlertIcon, AlertText, AlertTextGroup, AlertTitle } from './components';

export type AlertProps = React.ComponentPropsWithRef<'div'>;

const StyledAlert = styled(Flex)``;

function Alert({ children, ...props }: AlertProps) {
  return (
    <StyledAlert flexDirection="column" gap="medium" {...props}>
      {children}
    </StyledAlert>
  );
}

const AlertNamespace = Object.assign(Alert, {
  Text: AlertText,
  Title: AlertTitle,
  Icon: AlertIcon,
  TextGroup: AlertTextGroup,
});

export { AlertNamespace as Alert };
