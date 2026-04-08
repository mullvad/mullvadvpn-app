import { Accordion } from '../../../../../../../../../../../lib/components/accordion';
import type { ListItemTrailingActionsProps } from '../../../../../../../../../../../lib/components/list-item/components';
import { LocationAccordionTrailingAction } from './components';

export type LocationAccordionTrailingActionsProps = ListItemTrailingActionsProps;

function LocationAccordionTrailingActions({
  children,
  ...props
}: LocationAccordionTrailingActionsProps) {
  return <Accordion.Header.TrailingActions {...props}>{children}</Accordion.Header.TrailingActions>;
}

const LocationAccordionTrailingActionsNamespace = Object.assign(LocationAccordionTrailingActions, {
  Action: LocationAccordionTrailingAction,
});

export { LocationAccordionTrailingActionsNamespace as LocationAccordionTrailingActions };
