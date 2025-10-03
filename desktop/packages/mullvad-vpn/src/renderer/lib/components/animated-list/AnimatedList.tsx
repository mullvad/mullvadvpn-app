import { AnimatePresence } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { AnimatedListItem } from './components';

export type AnimatedListProps = React.ComponentPropsWithRef<'ul'>;

const StyledUl = styled.ul`
  width: 100%;
`;

function AnimatedList({ children, ...props }: AnimatedListProps) {
  return (
    <StyledUl {...props}>
      <AnimatePresence initial={false}>{children}</AnimatePresence>
    </StyledUl>
  );
}

const AnimatedListNamespace = Object.assign(AnimatedList, {
  Item: AnimatedListItem,
});

export { AnimatedListNamespace as AnimatedList };
