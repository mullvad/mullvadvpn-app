import { Accordion, type AccordionProps } from '../../../../lib/components/accordion';
import { LocationListItemAccordionProvider } from './LocationListItemAccordionContext';

export type LocationListItemAccordionProps = AccordionProps;

export function LocationListItemAccordion({ children, ...props }: LocationListItemAccordionProps) {
  return (
    <LocationListItemAccordionProvider>
      <Accordion {...props}>{children}</Accordion>
    </LocationListItemAccordionProvider>
  );
}
