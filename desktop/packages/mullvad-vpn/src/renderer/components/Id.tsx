import React, { useId } from 'react';

export type IdProps<T extends object> = T & {
  children: (id: string, props: Omit<T, 'children'>) => React.ReactNode;
};

export const Id = <T extends object>({ children, ...props }: IdProps<T>) => {
  const id = useId();
  return <React.Fragment>{children(id, props)}</React.Fragment>;
};
