import { Colors } from '../../../foundations';

export const useHoverColor = (color: Colors | undefined) => {
  switch (color) {
    case 'whiteAlpha60':
      return 'white';
    default:
      return undefined;
  }
};
