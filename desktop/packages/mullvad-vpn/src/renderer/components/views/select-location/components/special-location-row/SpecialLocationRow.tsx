import React, { useCallback } from 'react';

import { SpecialLocation } from '../../select-location-types';
import type { SpecialLocationRowInnerProps } from '../custom-exit-location-row';

export interface SpecialLocationRowProps<T> {
  source: SpecialLocation<T>;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: T) => void;
}

export function SpecialLocationRow<T>(props: SpecialLocationRowProps<T>) {
  const { onSelect: propsOnSelect } = props;
  const onSelect = useCallback(() => {
    if (!props.source.selected) {
      propsOnSelect(props.source.value);
    }
  }, [props.source, propsOnSelect]);

  const innerProps: SpecialLocationRowInnerProps<T> = {
    ...props,
    onSelect,
  };
  return <props.source.component {...innerProps} />;
}
