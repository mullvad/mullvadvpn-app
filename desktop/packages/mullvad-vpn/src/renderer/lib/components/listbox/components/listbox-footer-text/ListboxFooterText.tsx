import { Text, TextProps } from '../../../text';

export type ListboxFooterTextProps<E extends React.ElementType = 'span'> = TextProps<E>;

export const ListboxFooterText = <E extends React.ElementType = 'span'>(
  props: ListboxFooterTextProps<E>,
) => {
  return <Text variant="labelTiny" color="whiteAlpha60" {...props} />;
};
