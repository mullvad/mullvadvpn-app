import { Text, TextProps } from './Text';

export type TitleMediumProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const TitleMedium = <T extends React.ElementType = 'span'>(props: TitleMediumProps<T>) => (
  <Text variant="titleMedium" {...props} />
);
