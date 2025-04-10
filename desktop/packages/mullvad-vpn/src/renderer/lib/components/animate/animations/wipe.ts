import { css } from 'styled-components';

import { createAnimation } from '../utils';

export const wipeDownIn = createAnimation(
  'animation-wipe-down-in',
  css`
    from {
      display: none;
      max-height: 0;
    }
    to {
      display: block;
      max-height: min-content;
    }
  `,
);

export const wipeVerticalOut = createAnimation(
  'animation-wipe-vertical-out',
  css`
    from {
      display: block;
      max-height: min-content;
    }
    to {
      display: none;
      max-height: 0;
    }
  `,
);
