import styled from 'styled-components';

import { Accordion } from '../../../../lib/components/accordion';
import type { AccordionContentProps } from '../../../../lib/components/accordion/components';
import { useEffectScrollOnExpand } from './hooks';

export type LocationListItemAccordionContentProps = AccordionContentProps;

export const StyledLocationListItemAccordionContent = styled(Accordion.Content)``;

export const LocationListItemAccordionContent = (props: LocationListItemAccordionContentProps) => {
  useEffectScrollOnExpand();

  return <StyledLocationListItemAccordionContent {...props} />;
};
