import styled from 'styled-components';

import { Flex } from '../flex';
import {
  AlertIcon,
  AlertIconBadge,
  AlertList,
  AlertSpinner,
  AlertSubtitle,
  AlertText,
  AlertTextGroup,
  AlertTitle,
} from './components';

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
  Icon: AlertIcon,
  IconBadge: AlertIconBadge,
  List: AlertList,
  Spinner: AlertSpinner,
  Subtitle: AlertSubtitle,
  Text: AlertText,
  TextGroup: AlertTextGroup,
  Title: AlertTitle,
});

export { AlertNamespace as Alert };
