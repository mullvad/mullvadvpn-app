import { css } from 'styled-components';

export const createAnimationDeclaration = (animations: Array<{ name: string }>) => css`
  animation: ${animations
    .map(
      ({ name }) =>
        `${name} var(--duration) var(--timing-function) var(--direction) var(--iteration-count)`,
    )
    .join(', ')};
`;
