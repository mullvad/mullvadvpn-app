import styled from 'styled-components';

import { Colors, spacings } from '../../foundations';
import { TransientProps } from '../../types';
import { Flex } from '../flex';
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
  minHeight: '24px',
  height: '24px',
  padding: `${spacings.medium} 0`,
  gap: spacings.tiny,
});

const NavigationHeader = ({ titleVisible, children, ...props }: NavigationHeaderProps) => {
  return (
    <NavigationHeaderProvider titleVisible={!!titleVisible}>
      <StyledHeader {...props}>
        <Flex
          $flexDirection="column"
          $justifyContent="center"
          $padding={{
            horizontal: 'medium',
            vertical: 'small',
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
