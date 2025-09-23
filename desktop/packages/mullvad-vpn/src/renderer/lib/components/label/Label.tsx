import { Text } from '../text';
import { TextProps } from '../text';

export type LabelProps<T extends React.ElementType = 'label'> = TextProps<T>;

export function Label<T extends React.ElementType = 'label'>(props: LabelProps<T>) {
  return <Text as="label" {...props} />;
}
