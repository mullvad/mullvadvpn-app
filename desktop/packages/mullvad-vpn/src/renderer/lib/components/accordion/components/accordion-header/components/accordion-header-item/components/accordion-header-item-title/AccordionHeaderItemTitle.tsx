import { ListItem } from '../../../../../../../list-item';
import { TextProps } from '../../../../../../../text';
import { useAccordionContext } from '../../../../../../AccordionContext';

export type AccordionHeaderItemTitleProps = TextProps;

export function AccordionHeaderItemTitle({ children }: AccordionHeaderItemTitleProps) {
  const { titleId } = useAccordionContext();
  return <ListItem.Item.Label id={titleId}>{children}</ListItem.Item.Label>;
}
