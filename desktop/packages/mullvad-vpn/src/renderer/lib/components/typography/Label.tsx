import { Text } from './Text';
import { TextProps } from './Text';

export type LabelProps<T extends React.ElementType = 'label'> = TextProps<T>;

export const Label = <T extends React.ElementType = 'label'>(props: LabelProps<T>) => {
  return <Text as="label" {...props} />;
};
