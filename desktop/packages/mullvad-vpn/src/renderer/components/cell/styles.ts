export interface ButtonColors {
  $backgroundColor: string;
  $backgroundColorHover: string;
}

export const buttonColor = (props: ButtonColors) => {
  return {
    backgroundColor: props.$backgroundColor,
    '&&:not(:disabled):hover': {
      backgroundColor: props.$backgroundColorHover,
    },
  };
};
