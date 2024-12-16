import { Spacings } from '../variables';
import { LayoutSpacings } from './types';

export const all = (margin: Spacings) => ({ margin });

const vertical = (margin: Spacings) => ({
  ...top(margin),
  ...bottom(margin),
});

const horizontal = (margin: Spacings) => ({
  ...left(margin),
  ...right(margin),
});

const top = (marginTop: Spacings) => ({
  marginTop,
});

const right = (marginRight: Spacings) => ({
  marginRight,
});

const bottom = (marginBottom: Spacings) => ({
  marginBottom,
});

const left = (marginLeft: Spacings) => ({
  marginLeft,
});

export const margin: Record<keyof LayoutSpacings, (value: Spacings) => React.CSSProperties> = {
  all,
  vertical,
  horizontal,
  top,
  right,
  bottom,
  left,
};
