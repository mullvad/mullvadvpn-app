import { Colors } from '../../../foundations';

const colorMap: Record<
  Extract<Colors, 'chalk'> | 'default',
  {
    hover: Colors;
    active: Colors;
  }
> = {
  chalk: { hover: 'whiteAlpha60', active: 'whiteAlpha20' },
  default: { hover: 'whiteAlpha60', active: 'whiteAlpha20' },
} as const;

export const useStateColors = (
  color: Colors | undefined,
): {
  hover: Colors;
  active: Colors;
} => {
  if (color === 'chalk') {
    return colorMap[color];
  }
  return colorMap.default;
};
