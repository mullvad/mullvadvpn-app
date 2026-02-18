import { Accordion } from '../../lib/components/accordion';
import { ListItem } from '../../lib/components/list-item';
import {
  LocationListItemAccordion,
  LocationListItemAccordionContent,
  LocationListItemAccordionHeaderTitle,
  LocationListItemAccordionTrigger,
  LocationListItemHeader,
  LocationListItemIconButton,
} from './components';
import { LocationListItemProvider } from './LocationListItemContext';

export type LocationListItemProps = React.PropsWithChildren<{
  selected?: boolean;
}>;

function LocationListItem({ selected, children, ...props }: LocationListItemProps) {
  return (
    <LocationListItemProvider selected={selected} {...props}>
      {children}
    </LocationListItemProvider>
  );
}

const LocationListItemNamespace = Object.assign(LocationListItem, {
  Accordion: LocationListItemAccordion,
  AccordionTrigger: LocationListItemAccordionTrigger,
  AccordionContainer: Accordion.Container,
  AccordionContent: LocationListItemAccordionContent,
  HeaderChevron: Accordion.HeaderChevron,
  IconButton: LocationListItemIconButton,
  Header: LocationListItemHeader,
  HeaderActionGroup: Accordion.HeaderActionGroup,
  HeaderItem: Accordion.HeaderItem,
  HeaderTrigger: ListItem.Trigger,
  HeaderTrailingActions: Accordion.HeaderTrailingActions,
  HeaderTrailingAction: Accordion.HeaderTrailingAction,
  HeaderTitle: LocationListItemAccordionHeaderTitle,
});

export { LocationListItemNamespace as LocationListItem };
