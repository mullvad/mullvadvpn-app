import styled from 'styled-components';

import { Icon, IconProps } from '../../icon';
import { useAccordionContext } from '../AccordionContext';

export type AccordionIconProps = Omit<IconProps, 'icon'> & {
  icon?: IconProps['icon'];
};

export const StyledAccordionIcon = styled(Icon)`
  flex-shrink: 0;
`;

export function AccordionIcon({ icon, color = 'whiteAlpha80', ...props }: AccordionIconProps) {
  const { expanded: open } = useAccordionContext();
  const iconName = icon || (open ? 'chevron-up' : 'chevron-down');
  return <StyledAccordionIcon icon={iconName} color={color} {...props} />;
}
