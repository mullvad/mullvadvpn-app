import { Accordion } from '../../lib/components/accordion';
import { ListItem } from '../../lib/components/list-item';
import {
  LocationAccordionContent,
  LocationAccordionTitle,
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
  Accordion: Accordion,
  AccordionTrigger: Accordion.Trigger,
  AccordionContainer: Accordion.Container,
  AccordionContent: LocationAccordionContent,
  Icon: Accordion.Icon,
  IconButton: LocationListItemIconButton,
  Header: LocationListItemHeader,
  HeaderActionGroup: Accordion.HeaderActionGroup,
  HeaderItem: Accordion.HeaderItem,
  HeaderTrigger: ListItem.Trigger,
  HeaderTrailingActions: Accordion.HeaderTrailingActions,
  HeaderTrailingAction: Accordion.HeaderTrailingAction,
  Title: LocationAccordionTitle,
});

export { LocationListItemNamespace as LocationListItem };
