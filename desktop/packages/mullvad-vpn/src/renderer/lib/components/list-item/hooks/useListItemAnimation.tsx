import { css, keyframes } from 'styled-components';

import { colors } from '../../../foundations';
import { ListItemAnimation } from '../ListItem';

const flash = keyframes`
  0%   { background-color: var(--background-color) }
  50%  { background-color: ${colors.whiteOnBlue20} }
  100% { background-color: var(--background-color) }
`;

const dim = keyframes`
  0%   { opacity: 100% }
  25%  { opacity: 50% }
  50%  { opacity: 50% }
  75%  { opacity: 50% }
  100% { opacity: 100% }
`;

export const useListItemAnimation = (animation: ListItemAnimation | undefined) => {
  if (animation === 'flash') {
    return css`
      animation: ${flash} 0.75s ease-in-out 0s 2 normal forwards;
    `;
  }
  if (animation === 'dim') {
    return css`
      animation: ${dim} 1.5s ease-in-out 0s 1 normal forwards;
    `;
  }
  return undefined;
};
