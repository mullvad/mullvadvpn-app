import React from 'react';
import styled from 'styled-components';

import type { PolymorphicProps } from '../../types';

export type TriggerProps<T extends React.ElementType = 'button'> = PolymorphicProps<T>;

const StyledTrigger = styled.button`
  cursor: default;
  text-decoration: none;
`;

export function Trigger<T extends React.ElementType = 'button'>({ as, ...props }: TriggerProps<T>) {
  const { onClick, ...anchorProps } = props;
  const handleClick = React.useCallback(
    (event: React.MouseEvent<HTMLAnchorElement>) => {
      event.preventDefault();
      onClick?.(event);
    },
    [onClick],
  );

  if (as == 'a') {
    return <StyledTrigger as="a" tabIndex={0} href="" onClick={handleClick} {...anchorProps} />;
  }
  return <StyledTrigger as={as} tabIndex={0} {...props} />;
}
