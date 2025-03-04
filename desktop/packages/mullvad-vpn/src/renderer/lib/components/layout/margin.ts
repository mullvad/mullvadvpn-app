import { Spacings, spacings } from '../../foundations';
import { LayoutSpacings } from './types';

export const all = (margin: Spacings) => {
  const marginAll = spacings[margin];
  return { margin: marginAll };
};

const vertical = (margin: Spacings) => ({
  ...top(margin),
  ...bottom(margin),
});

const horizontal = (margin: Spacings) => {
  return {
    ...left(margin),
    ...right(margin),
  };
};

const top = (margin: Spacings) => {
  const marginTop = spacings[margin];
  return {
    marginTop,
  };
};

const right = (margin: Spacings) => {
  const marginRight = spacings[margin];
  return {
    marginRight,
  };
};

const bottom = (margin: Spacings) => {
  const marginBottom = spacings[margin];
  return {
    marginBottom,
  };
};

const left = (margin: Spacings) => {
  const marginLeft = spacings[margin];
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
