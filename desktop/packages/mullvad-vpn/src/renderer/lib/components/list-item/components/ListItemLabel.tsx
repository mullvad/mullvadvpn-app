import { Colors } from '../../../foundations';
import { LabelTinyProps, TitleMedium } from '../../typography';
import { useListItem } from '../ListItemContext';

export type ListItemLabelProps = LabelTinyProps;

export const ListItemLabel = ({ children, ...props }: ListItemLabelProps) => {
  const { disabled } = useListItem();
  return (
    <TitleMedium color={disabled ? Colors.white40 : Colors.white} {...props}>
      {children}
    </TitleMedium>
  );
};
