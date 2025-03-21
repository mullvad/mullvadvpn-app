import { Colors } from '../../../foundations';
import { Text, TextProps } from '../../typography';
import { useListItem } from '../ListItemContext';

export type ListItemProps<E extends React.ElementType = 'span'> = Omit<TextProps<E>, 'variant'> & {
  variant?: Extract<TextProps<E>['variant'], 'labelTiny' | 'footnoteMini'>;
};

export const ListItemText = ({ variant = 'labelTiny', children, ...props }: ListItemProps) => {
  const { disabled } = useListItem();
  return (
    <Text variant={variant} color={disabled ? Colors.white40 : Colors.white60} {...props}>
      {children}
    </Text>
  );
};
