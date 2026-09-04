import { Text, TextProps } from '../../../../../../../text';

export type WizardSlideTextProps = TextProps;

export function WizardSlideText({ children, ...props }: WizardSlideTextProps) {
  return (
    <Text variant="bodySmall" color="whiteAlpha60" {...props}>
      {children}
    </Text>
  );
}
