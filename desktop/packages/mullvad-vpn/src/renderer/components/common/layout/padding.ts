import { Spacings, spacings } from '../../../tokens';
import { LayoutSpacings } from './types';

export const p = (value: Spacings) => {
  return `padding: ${spacings[value]};`;
};

const y = (value: Spacings) => {
  return `padding-top: ${spacings[value]};
      padding-bottom: ${spacings[value]};
      `;
};

const x = (value: Spacings) => {
  return `padding-left: ${spacings[value]};
        padding-right: ${spacings[value]};
        `;
};

const top = (value: Spacings) => `padding-top: ${spacings[value]};`;
const right = (value: Spacings) => `padding-right: ${spacings[value]};`;
const bottom = (value: Spacings) => `padding-bottom: ${spacings[value]};`;
const left = (value: Spacings) => `padding-left: ${spacings[value]};`;

export const padding: Record<keyof LayoutSpacings, (value: Spacings) => string | undefined> = {
  y,
  x,
  top,
  right,
  bottom,
  left,
};
