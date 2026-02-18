import { IconProps } from '../../icon';
import { ListItem } from '../../list-item';
import { useAccordionContext } from '../AccordionContext';

export type AccordionHeaderChevronProps = Omit<IconProps, 'icon'> & {
  icon?: IconProps['icon'];
};

export function AccordionHeaderChevron({ icon, ...props }: AccordionHeaderChevronProps) {
  const { expanded } = useAccordionContext();
  const iconName = icon || (expanded ? 'chevron-up' : 'chevron-down');
  return <ListItem.Icon icon={iconName} {...props} />;
}
