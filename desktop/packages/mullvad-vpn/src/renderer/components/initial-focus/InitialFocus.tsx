import React from 'react';

import { useInitialFocus } from '../../hooks';

type AnyElement = React.ElementType;

export type InitialFocusProps<E extends AnyElement> = {
  children: React.ReactElement<React.ComponentPropsWithRef<E>>;
} & Omit<React.ComponentPropsWithoutRef<E>, 'children'>;

export function InitialFocus<E extends AnyElement>({ children, ...props }: InitialFocusProps<E>) {
  const { ref } = useInitialFocus();

  console.log('ref', ref);

  const refCallback = (element) => {
    console.log('ref mounted');
    if (ref) {
      ref.current = element;
    }
  };

  return React.cloneElement(children, {
    ref: refCallback,
    tabIndex: -1,
    ...props,
  } as React.ComponentPropsWithRef<E>);
}
