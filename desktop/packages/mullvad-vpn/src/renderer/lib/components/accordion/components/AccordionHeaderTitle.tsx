import { ListItem } from '../../list-item';
import { TextProps } from '../../text';
import { useAccordionContext } from '../AccordionContext';

export type AccordionHeaderTitleProps = TextProps;

export function AccordionHeaderTitle({ children }: AccordionHeaderTitleProps) {
  const { titleId } = useAccordionContext();
  return <ListItem.Item.Label id={titleId}>{children}</ListItem.Item.Label>;
}
