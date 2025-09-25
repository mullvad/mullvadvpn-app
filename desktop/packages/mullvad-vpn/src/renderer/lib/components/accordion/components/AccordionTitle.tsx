import { ListItem } from '../../list-item';
import { TextProps } from '../../text';
import { useAccordionContext } from '../AccordionContext';

export type AccordionTitleProps = TextProps;

export function AccordionTitle({ children }: AccordionTitleProps) {
  const { titleId } = useAccordionContext();
  return <ListItem.Label id={titleId}>{children}</ListItem.Label>;
}
