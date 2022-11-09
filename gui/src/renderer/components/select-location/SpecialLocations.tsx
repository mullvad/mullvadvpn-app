import React from 'react';

import { SpecialLocation } from './SpecialLocation';

interface ISpecialLocationsProps<T> {
  children: React.ReactNode;
  selectedValue?: T;
  selectedElementRef?: React.Ref<SpecialLocation<T>>;
  onSelect?: (value: T) => void;
}

export function SpecialLocations<T>(props: ISpecialLocationsProps<T>) {
  return (
    <>
      {React.Children.map(props.children, (child) => {
        if (React.isValidElement(child) && child.type === SpecialLocation) {
          const isSelected = props.selectedValue === child.props.value;

          return React.cloneElement(child, {
            ...child.props,
            forwardedRef: isSelected ? props.selectedElementRef : undefined,
            onSelect: props.onSelect,
            isSelected,
          });
        } else {
          return undefined;
        }
      })}
    </>
  );
}
