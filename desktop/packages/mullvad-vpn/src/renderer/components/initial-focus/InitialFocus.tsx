import React from 'react';

import { useInitialFocus } from '../../hooks';

type AnyElement = React.ElementType;

export type InitialFocusProps<E extends AnyElement> = {
  preventFocus?: boolean;
  children: React.ReactElement<React.ComponentPropsWithRef<E>>;
} & Omit<React.ComponentPropsWithoutRef<E>, 'children'>;

export function InitialFocus<E extends AnyElement>({
  preventFocus,
  children,
  ...props
}: InitialFocusProps<E>) {
  const { ref } = useInitialFocus(!preventFocus);
  return React.cloneElement(children, {
    ref,
    tabIndex: -1,
    ...props,
  } as React.ComponentPropsWithRef<E>);
}
