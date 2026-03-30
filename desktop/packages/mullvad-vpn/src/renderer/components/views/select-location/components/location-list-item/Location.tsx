import { LocationAccordion, LocationListItem } from './components';
import { LocationProvider } from './LocationContext';

export type LocationProps = React.PropsWithChildren<{
  root?: boolean;
  selected?: boolean;
}>;

function Location({ root, selected, children, ...props }: LocationProps) {
  return (
    <LocationProvider root={root} selected={selected} {...props}>
      {children}
    </LocationProvider>
  );
}

const LocationNamespace = Object.assign(Location, {
  Accordion: LocationAccordion,
  ListItem: LocationListItem,
});

export { LocationNamespace as Location };
