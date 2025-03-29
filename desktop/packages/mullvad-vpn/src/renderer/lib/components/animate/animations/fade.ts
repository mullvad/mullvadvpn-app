import { css } from 'styled-components';

import { createAnimation } from '../utils';

export const fadeIn = createAnimation(
  'animation-fade-in',
  css`
    from {
      display: none;
      opacity: 0;
    }
    to {
      display: block;
      opacity: 1;
    }
  `,
);

export const fadeOut = createAnimation(
  'animation-fade-out',
  css`
    from {
      display: block;
      opacity: 1;
    }
    to {
      display: none;
      opacity: 0;
    }
  `,
);
