import styled from 'styled-components';

import { Colors } from '../../foundations';
import { TransientProps } from '../../types';
import { Flex } from '../flex';
import { MainHeaderIconButton } from './components';

export type HeaderProps = React.PropsWithChildren<{
  size?: '1' | '2';
  variant?: 'default' | 'success' | 'error';
}>;

const sizes = {
  '1': '68px',
  '2': '80px',
};

const variants = {
  default: Colors.blue,
  error: Colors.red,
  success: Colors.green,
};

const StyledHeader = styled.header<TransientProps<HeaderProps>>(
  ({ $size = '1', $variant = 'default' }) => ({
    height: sizes[$size],
    minHeight: sizes[$size],

    backgroundColor: variants[$variant],
    transition: 'height 250ms ease-in-out, min-height 250ms ease-in-out',
  }),
);

const MainHeader = ({ size = '1', variant = 'default', children, ...props }: HeaderProps) => {
  return (
    <StyledHeader $size={size} $variant={variant} {...props}>
      <Flex
        $flexDirection="column"
        $justifyContent="center"
        $margin={{
          horizontal: 'medium',
          top: 'medium',
          bottom: 'small',
        }}>
        {children}
      </Flex>
    </StyledHeader>
  );
};

const MainHeaderNamespace = Object.assign(MainHeader, {
  IconButton: MainHeaderIconButton,
});

export { MainHeaderNamespace as MainHeader };
