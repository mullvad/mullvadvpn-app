import { KnownTarget } from 'styled-components/dist/types';

import { Text, TextProps } from './Text';
export type TitleBigProps<T extends KnownTarget> = TextProps<T>;

export const TitleBig = <T extends KnownTarget>({ ...props }: TitleBigProps<T>) => (
  <Text variant="titleBig" {...props} />
);
