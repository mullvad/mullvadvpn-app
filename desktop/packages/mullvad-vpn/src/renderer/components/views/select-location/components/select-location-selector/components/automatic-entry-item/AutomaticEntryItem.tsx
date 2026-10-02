import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorButtonItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import { useAutomaticLocationName } from '../../hooks';

export type AutomaticEntryItemProps = Omit<LocationSelectorButtonItemProps, 'id' | 'type'>;

export function AutomaticEntryItem(props: AutomaticEntryItemProps) {
  const label = useAutomaticLocationName();

  return (
    <LocationSelector.Items.ButtonItem id="entryAutomatic" type="entryAutomatic" {...props}>
      <LocationSelector.Items.ButtonItem.Button label={label} />
    </LocationSelector.Items.ButtonItem>
  );
}
