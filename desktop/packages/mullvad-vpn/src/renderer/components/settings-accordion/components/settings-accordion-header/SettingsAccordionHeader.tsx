import styled from 'styled-components';

import { useScrollToListItem } from '../../../../hooks';
import {
  AccordionHeader,
  AccordionHeaderProps,
} from '../../../../lib/components/accordion/components';
import { useSettingsAccordionContext } from '../../SettingsAccordionContext';

export type SettingsAccordionHeaderProps = AccordionHeaderProps;

export const StyledSettingsAccordionHeader = styled(AccordionHeader)``;

export function SettingsAccordionHeader({ children, ...props }: SettingsAccordionHeaderProps) {
  const { anchorId } = useSettingsAccordionContext();
  const { ref, animation } = useScrollToListItem(anchorId);
  return (
    <StyledSettingsAccordionHeader ref={ref} animation={animation} {...props}>
      {children}
    </StyledSettingsAccordionHeader>
  );
}
