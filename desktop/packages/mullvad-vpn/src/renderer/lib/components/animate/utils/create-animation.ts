import { css, RuleSet } from 'styled-components';

export const createAnimation = (name: string, frames: RuleSet) => ({
  name,
  rule: css`
    @keyframes ${name} {
      ${frames}
    }
  `,
});
