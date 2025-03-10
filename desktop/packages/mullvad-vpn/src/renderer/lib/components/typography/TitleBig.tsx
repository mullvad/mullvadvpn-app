import { WebTarget } from 'styled-components';

import { Text, TextProps } from './Text';
export type TitleBigProps = Omit<TextProps<WebTarget>, 'variant'>;

export const TitleBig = ({ children, ...props }: TitleBigProps) => (
  <Text variant="titleBig" {...props}>
    {children}
  </Text>
);
