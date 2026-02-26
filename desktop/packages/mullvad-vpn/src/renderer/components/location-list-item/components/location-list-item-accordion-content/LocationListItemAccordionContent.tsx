import { Accordion } from '../../../../lib/components/accordion';
import type { AccordionContentProps } from '../../../../lib/components/accordion/components';
import { useEffectScrollOnExpand } from './hooks';

export type LocationListItemAccordionContentProps = AccordionContentProps;

export const LocationListItemAccordionContent = (props: LocationListItemAccordionContentProps) => {
  useEffectScrollOnExpand();

  return <Accordion.Content {...props} />;
};
