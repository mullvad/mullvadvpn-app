import React from 'react';

export function useChildrenValues(children: React.ReactNode[]): string[] {
  return React.useMemo(() => {
    const values: string[] = [];

    React.Children.forEach(children, (child) => {
      if (React.isValidElement<{ value?: string }>(child)) {
        if ('value' in child.props && typeof child.props.value !== 'undefined') {
          values.push(child.props.value);
        }
      }
    });

    return values;
  }, [children]);
}
