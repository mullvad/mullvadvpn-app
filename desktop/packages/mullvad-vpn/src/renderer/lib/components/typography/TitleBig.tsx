import { Text, TextProps } from './Text';

export type TitleBigProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const TitleBig = <T extends React.ElementType = 'span'>(props: TitleBigProps<T>) => (
  <Text variant="titleBig" {...props} />
);
