import { Spacings } from '../../foundations';
import { LayoutSpacings } from './types';

export const all = (padding: Spacings) => ({ padding });

const vertical = (padding: Spacings) => ({
  ...top(padding),
  ...bottom(padding),
});

const horizontal = (padding: Spacings) => ({
  ...left(padding),
  ...right(padding),
});

const top = (paddingTop: Spacings) => ({
  paddingTop,
});

const right = (paddingRight: Spacings) => ({
  paddingRight,
});

const bottom = (paddingBottom: Spacings) => ({
  paddingBottom,
});

const left = (paddingLeft: Spacings) => ({
  paddingLeft,
});

export const padding: Record<keyof LayoutSpacings, (value: Spacings) => React.CSSProperties> = {
  all,
  vertical,
  horizontal,
  top,
  right,
  bottom,
  left,
};
