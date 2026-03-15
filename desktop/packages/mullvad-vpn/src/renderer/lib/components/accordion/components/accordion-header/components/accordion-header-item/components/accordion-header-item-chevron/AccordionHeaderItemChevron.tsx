import { IconProps } from '../../../../../../../icon';
import { ListItem } from '../../../../../../../list-item';
import { useAccordionContext } from '../../../../../../AccordionContext';

export type AccordionHeaderItemChevronProps = Omit<IconProps, 'icon'> & {
  icon?: IconProps['icon'];
};

export function AccordionHeaderItemChevron({ icon, ...props }: AccordionHeaderItemChevronProps) {
  const { expanded } = useAccordionContext();
  const iconName = icon || (expanded ? 'chevron-up' : 'chevron-down');
  return <ListItem.Item.Icon icon={iconName} {...props} />;
}
