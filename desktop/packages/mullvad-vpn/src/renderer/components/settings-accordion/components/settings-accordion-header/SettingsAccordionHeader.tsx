import styled from 'styled-components';

import { useScrollToListItem } from '../../../../hooks';
import {
  AccordionHeader,
  type AccordionHeaderProps,
} from '../../../../lib/components/accordion/components';
import { useSettingsAccordionContext } from '../../SettingsAccordionContext';

export type SettingsAccordionHeaderProps = AccordionHeaderProps;

export const StyledSettingsAccordionHeader = styled(AccordionHeader)``;

function SettingsAccordionHeader({ children, ...props }: SettingsAccordionHeaderProps) {
  const { anchorId } = useSettingsAccordionContext();
  const { ref, animation } = useScrollToListItem(anchorId);
  return (
    <StyledSettingsAccordionHeader ref={ref} animation={animation} {...props}>
      {children}
    </StyledSettingsAccordionHeader>
  );
}

const SettingsAccordionHeaderNamespace = Object.assign(SettingsAccordionHeader, {
  Item: AccordionHeader.Item,
  AccordionTrigger: AccordionHeader.AccordionTrigger,
});

export { SettingsAccordionHeaderNamespace as SettingsAccordionHeader };
