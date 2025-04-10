import { css, RuleSet } from 'styled-components';

import { Animation } from '../Animate';
import { animations } from '../animations';
import { createAnimationDeclaration } from '../utils';

export const useAnimations = (values: Animation[]) => {
  const inAnimations: Array<{ name: string; rule: RuleSet }> = [];
  const outAnimations: Array<{ name: string; rule: RuleSet }> = [];

  values.forEach((animation) => {
    if (animation.type === 'fade') {
      inAnimations.push(animations.fade.in);
      outAnimations.push(animations.fade.out);
    } else if (animation.type === 'wipe' && animation.direction === 'vertical') {
      inAnimations.push(animations.wipeDown.in);
      outAnimations.push(animations.wipeDown.out);
    }
  });

  return css`
    ${inAnimations.map((animation) => animation.rule)}
    ${outAnimations.map((animation) => animation.rule)}
    ${createAnimationDeclaration(outAnimations)}
    &&[data-present='true'] {
      ${createAnimationDeclaration(inAnimations)}
    }
  `;
};
