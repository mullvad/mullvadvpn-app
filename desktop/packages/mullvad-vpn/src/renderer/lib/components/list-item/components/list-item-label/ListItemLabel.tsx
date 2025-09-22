import { LabelTinySemiBoldProps, TitleMedium } from '../../../typography';
import { useListItemContext } from '../../ListItemContext';

export type ListItemLabelProps<E extends React.ElementType = 'span'> = LabelTinySemiBoldProps<E>;

export const ListItemLabel = <E extends React.ElementType = 'span'>(
  props: ListItemLabelProps<E>,
) => {
  const { disabled } = useListItemContext();
  return <TitleMedium color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
};
