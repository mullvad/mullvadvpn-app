import styled from 'styled-components';

import { Animate } from '../../animate';
import { useAccordionContext } from '../AccordionContext';

export type AccordionContentProps = {
  children?: React.ReactNode;
};

const StyledAccordionContent = styled.div`
  width: 100%;
`;

export function AccordionContent({ children }: AccordionContentProps) {
  const { contentId, triggerId, expanded } = useAccordionContext();
  return (
    <Animate
      present={expanded}
      animations={[{ type: 'wipe', direction: 'vertical' }]}
      duration="0.35s">
      <StyledAccordionContent id={contentId} aria-labelledby={triggerId} role="region">
        {children}
      </StyledAccordionContent>
    </Animate>
  );
}
