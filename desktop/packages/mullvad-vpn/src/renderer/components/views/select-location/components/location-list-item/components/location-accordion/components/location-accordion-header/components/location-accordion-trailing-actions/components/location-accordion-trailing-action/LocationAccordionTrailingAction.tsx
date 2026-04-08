import { Accordion } from '../../../../../../../../../../../../../lib/components/accordion';
import type { ListItemTrailingActionProps } from '../../../../../../../../../../../../../lib/components/list-item/components/list-item-trailing-actions/components';

export type LocationAccordionTrailingActionProps = ListItemTrailingActionProps;

function LocationAccordionTrailingAction({
  children,
  ...props
}: LocationAccordionTrailingActionProps) {
  return (
    <Accordion.Header.TrailingActions.Action {...props}>
      {children}
    </Accordion.Header.TrailingActions.Action>
  );
}

const LocationAccordionTrailingActionNamespace = Object.assign(LocationAccordionTrailingAction, {
  Chevron: Accordion.Header.Item.Chevron,
});

export { LocationAccordionTrailingActionNamespace as LocationAccordionTrailingAction };
