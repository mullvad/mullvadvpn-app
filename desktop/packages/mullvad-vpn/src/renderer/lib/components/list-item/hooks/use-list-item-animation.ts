import { css, keyframes } from 'styled-components';

import { colors } from '../../../foundations';
import { ListItemAnimation } from '../ListItem';

const flash = keyframes`
  from   { background-color: var(--background-color) }
  to  { background-color: ${colors.whiteOnBlue20} }
`;

const dim = keyframes`
  0%   { opacity: 100% }
  10%  { opacity: 50% }
  50%  { opacity: 50% }
  90%  { opacity: 50% }
  100% { opacity: 100% }
`;

export const useListItemAnimation = (animation?: ListItemAnimation | false) => {
  const flashDuration = 200;
  const flashDelay = 450;
  const dimDuration = (flashDelay + flashDuration * 4) * 1.1;
  if (animation === 'flash') {
    return css`
      animation: ${flash} ${flashDuration}ms ease-in-out ${flashDelay}ms 4 alternate;
    `;
  }
  if (animation === 'dim') {
    return css`
      animation: ${dim} ${dimDuration}ms ease-in-out 0ms normal;
    `;
  }
  return undefined;
};
