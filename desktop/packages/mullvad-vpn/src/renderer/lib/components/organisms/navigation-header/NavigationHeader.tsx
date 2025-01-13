import styled from 'styled-components';

import { Colors, Spacings } from '../../../foundations';
import { TransientProps } from '../../../types';
import { Flex } from '../../layout';
import {
  NavigationHeaderButtonGroup,
  NavigationHeaderIconButton,
  NavigationHeaderTitle,
} from './components';
import { NavigationHeaderProvider } from './NavigationHeaderContext';

export type NavigationHeaderProps = React.PropsWithChildren<{
  titleVisible?: boolean;
}>;

const StyledHeader = styled.nav<TransientProps<NavigationHeaderProps>>({
  backgroundColor: Colors.darkBlue,
});

export const StyledContent = styled.div({
  display: 'grid',
  gridTemplateColumns: '1fr auto 1fr',
  placeContent: 'center',
  minHeight: '32px',
  height: '32px',
});

const NavigationHeader = ({ titleVisible, children, ...props }: NavigationHeaderProps) => {
  return (
    <NavigationHeaderProvider titleVisible={!!titleVisible}>
      <StyledHeader {...props}>
        <Flex
          $flexDirection="column"
          $justifyContent="center"
          $padding={{
            horizontal: Spacings.spacing5,
            vertical: Spacings.spacing3,
          }}>
          <StyledContent>{children}</StyledContent>
        </Flex>
      </StyledHeader>
    </NavigationHeaderProvider>
  );
};

const NavigationHeaderNamespace = Object.assign(NavigationHeader, {
  ButtonGroup: NavigationHeaderButtonGroup,
  IconButton: NavigationHeaderIconButton,
  Title: NavigationHeaderTitle,
});

export { NavigationHeaderNamespace as NavigationHeader };
