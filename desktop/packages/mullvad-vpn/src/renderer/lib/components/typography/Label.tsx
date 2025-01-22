import { Text } from './Text';
import { TextProps } from './Text';

export type LabelProps = TextProps & React.LabelHTMLAttributes<HTMLLabelElement>;

export const Label = ({ children, ...props }: LabelProps) => {
  return (
    <Text as={'label'} {...props}>
      {children}
    </Text>
  );
};
