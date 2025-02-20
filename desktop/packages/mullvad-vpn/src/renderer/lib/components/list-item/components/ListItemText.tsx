import { WebTarget } from 'styled-components';

import { Colors } from '../../../foundations';
import { Text, TextProps } from '../../typography';
import { useListItem } from '../ListItemContext';

export type ListItemProps<T extends WebTarget = 'span'> = Omit<TextProps<T>, 'variant'> & {
  variant?: Extract<TextProps<T>['variant'], 'labelTiny' | 'footnoteMini'>;
};

export const ListItemText = ({ variant = 'labelTiny', children, ...props }: ListItemProps) => {
  const { disabled } = useListItem();
  return (
    <Text variant={variant} color={disabled ? Colors.white40 : Colors.white60} {...props}>
      {children}
    </Text>
  );
};
