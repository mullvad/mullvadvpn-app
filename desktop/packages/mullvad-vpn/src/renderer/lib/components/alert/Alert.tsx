import styled from 'styled-components';

import { colors } from '../../foundations';
import { Flex } from '../flex';
import { AlertIcon, AlertText, AlertTextGroup, AlertTitle } from './components';

export type AlertProps = React.ComponentPropsWithRef<'div'>;

const StyledAlert = styled(Flex)`
  background-color: ${colors.darkBlue};
`;

function Alert({ children, ...props }: AlertProps) {
  return (
    <StyledAlert $flexDirection="column" $padding="large" $gap="medium" {...props}>
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
