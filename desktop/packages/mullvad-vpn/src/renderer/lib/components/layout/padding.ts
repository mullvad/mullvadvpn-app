import { Spacings, spacings } from '../../foundations';
import { LayoutSpacings } from './types';

export const all = (padding: Spacings) => {
  const paddingAll = spacings[padding];
  return { padding: paddingAll };
};

const vertical = (padding: Spacings) => ({
  ...top(padding),
  ...bottom(padding),
});

const horizontal = (padding: Spacings) => ({
  ...left(padding),
  ...right(padding),
});

const top = (padding: Spacings) => {
  const paddingTop = spacings[padding];
  return {
    paddingTop,
  };
};

const right = (padding: Spacings) => {
  const paddingRight = spacings[padding];
  return {
    paddingRight,
  };
};

const bottom = (padding: Spacings) => {
  const paddingBottom = spacings[padding];
  return {
    paddingBottom,
  };
};

const left = (padding: Spacings) => {
  const paddingLeft = spacings[padding];
  return {
    paddingLeft,
  };
};

export const padding: Record<keyof LayoutSpacings, (value: Spacings) => React.CSSProperties> = {
  all,
  vertical,
  horizontal,
  top,
  right,
  bottom,
  left,
};
