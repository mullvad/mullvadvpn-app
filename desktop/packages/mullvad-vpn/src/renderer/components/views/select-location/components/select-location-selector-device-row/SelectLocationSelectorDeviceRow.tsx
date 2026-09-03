import { messages } from '../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../lib/components/location-selector';

export function SelectLocationSelectorDeviceRow() {
  return (
    <LocationSelector.Row position="top">
      <LocationSelector.Row.Content>
        <LocationSelector.Row.Icon icon="device" />
        <LocationSelector.Row.Label>{messages.gettext('Your device')}</LocationSelector.Row.Label>
      </LocationSelector.Row.Content>
    </LocationSelector.Row>
  );
}
