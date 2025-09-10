import { Colors } from '../../../foundations';

const colorMap: Record<
  Extract<Colors, 'white' | 'whiteAlpha60'>,
  {
    hover: Colors;
    active: Colors;
  }
> = {
  whiteAlpha60: { hover: 'chalk', active: 'white' },
  white: { hover: 'chalk', active: 'whiteAlpha60' },
} as const;

export const useStateColors = (
  color: Colors | undefined,
): {
  hover: Colors;
  active: Colors;
} => {
  if (color === 'white' || color === 'whiteAlpha60') {
    return colorMap[color];
  }
  return colorMap.white;
};
