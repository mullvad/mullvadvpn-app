import { Text } from './Text';
import { TextProps } from './Text';

export type LabelProps = TextProps<'label'>;

export const Label = ({ children, ...props }: LabelProps) => {
  return (
    <Text as={'label'} {...props}>
      {children}
    </Text>
  );
};
