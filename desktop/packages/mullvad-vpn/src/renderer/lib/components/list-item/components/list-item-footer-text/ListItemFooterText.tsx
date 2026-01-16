import { Text, TextProps } from '../../../text';

export type ListItemFooterTextProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListItemFooterText = <E extends React.ElementType = 'span'>(
  props: ListItemFooterTextProps<E>,
) => {
  return <Text variant="labelTiny" color="whiteAlpha60" {...props} />;
};
