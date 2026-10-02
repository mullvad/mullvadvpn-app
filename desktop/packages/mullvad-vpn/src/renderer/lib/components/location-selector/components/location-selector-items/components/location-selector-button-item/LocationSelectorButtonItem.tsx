import type { LocationSelectorSelectedItem } from '../../../../LocationSelector';
import type { LocationSelectorItemType } from '../../types';
import { LocationSelectorItem, type LocationSelectorItemProps } from '../location-selector-item';
import { LocationSelectorButton } from './components';
import { LocationSelectorButtonItemProvider } from './LocationSelectorButtonItemContext';

export type LocationSelectorButtonItemProps = LocationSelectorItemProps & {
  id: LocationSelectorSelectedItem;
  type: LocationSelectorItemType;
};

function LocationSelectorButtonItem({
  id,
  type,
  children,
  ...props
}: LocationSelectorButtonItemProps) {
  return (
    <LocationSelectorButtonItemProvider id={id} type={type}>
      <LocationSelectorItem {...props}>{children}</LocationSelectorItem>
    </LocationSelectorButtonItemProvider>
  );
}

const LocationSelectorButtonItemNamespace = Object.assign(LocationSelectorButtonItem, {
  Button: LocationSelectorButton,
});

export { LocationSelectorButtonItemNamespace as LocationSelectorButtonItem };
