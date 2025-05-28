export type Animation = FadeAnimation | WipeAnimation;

export type FadeAnimation = {
  type: 'fade';
};

export type WipeAnimation = {
  type: 'wipe';
  direction: 'vertical';
};
