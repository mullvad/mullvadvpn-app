import { LocationListItemAccordion } from './components';
import { LocationListItemProvider } from './LocationListItemContext';

export type LocationListItemProps = React.PropsWithChildren<{
  root?: boolean;
  selected?: boolean;
}>;

function LocationListItem({ root, selected, children, ...props }: LocationListItemProps) {
  return (
    <LocationListItemProvider root={root} selected={selected} {...props}>
      {children}
    </LocationListItemProvider>
  );
}

const LocationListItemNamespace = Object.assign(LocationListItem, {
  Accordion: LocationListItemAccordion,
});

export { LocationListItemNamespace as LocationListItem };
