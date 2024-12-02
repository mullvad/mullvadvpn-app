import { Spacings, spacings } from '../../../tokens';
import { LayoutSpacings } from './types';

export const m = (value: Spacings) => {
  return `margin: ${spacings[value]};`;
};

const y = (value: Spacings) => {
  return `margin-top: ${spacings[value]};
      margin-bottom: ${spacings[value]};
      `;
};

const x = (value: Spacings) => {
  return `margin-left: ${spacings[value]};
        margin-right: ${spacings[value]};
        `;
};

const top = (value: Spacings) => `margin-top: ${spacings[value]};`;
const right = (value: Spacings) => `margin-right: ${spacings[value]};`;
const bottom = (value: Spacings) => `margin-bottom: ${spacings[value]};`;
const left = (value: Spacings) => `margin-left: ${spacings[value]};`;

export const margin: Record<keyof LayoutSpacings, (value: Spacings) => string | undefined> = {
  y,
  x,
  top,
  right,
  bottom,
  left,
};
