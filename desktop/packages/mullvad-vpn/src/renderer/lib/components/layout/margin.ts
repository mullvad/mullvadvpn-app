import { Spacings, spacings } from '../../foundations';
import { LayoutSpacings } from './types';

export const all = (value: Spacings) => {
  const marginAll = spacings[value];
  return { margin: marginAll };
};

const vertical = (value: Spacings) => ({
  ...top(value),
  ...bottom(value),
});

const horizontal = (value: Spacings) => {
  return {
    ...left(value),
    ...right(value),
  };
};

const top = (value: Spacings) => {
  const marginTop = spacings[value];
  return {
    marginTop,
  };
};

const right = (value: Spacings) => {
  const marginRight = spacings[value];
  return {
    marginRight,
  };
};

const bottom = (value: Spacings) => {
  const marginBottom = spacings[value];
  return {
    marginBottom,
  };
};

const left = (value: Spacings) => {
  const marginLeft = spacings[value];
  return {
    marginLeft,
  };
};

export const margin: Record<keyof LayoutSpacings, (value: Spacings) => React.CSSProperties> = {
  all,
  vertical,
  horizontal,
  top,
  right,
  bottom,
  left,
};
