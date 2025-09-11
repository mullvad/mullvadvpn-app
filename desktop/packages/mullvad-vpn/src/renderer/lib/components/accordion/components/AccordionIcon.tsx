import { IconProps } from '../../icon';
import { ListItem } from '../../list-item';
import { useAccordionContext } from '../AccordionContext';

export type AccordionIconProps = Omit<IconProps, 'icon'> & {
  icon?: IconProps['icon'];
};

export function AccordionIcon({ icon, ...props }: AccordionIconProps) {
  const { expanded } = useAccordionContext();
  const iconName = icon || (expanded ? 'chevron-up' : 'chevron-down');
  return <ListItem.Icon icon={iconName} {...props} />;
}
