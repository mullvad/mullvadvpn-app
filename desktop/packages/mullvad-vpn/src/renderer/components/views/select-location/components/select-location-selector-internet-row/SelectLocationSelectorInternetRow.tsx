import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { LocationSelector } from '../../../../../lib/components/location-selector';

export function SelectLocationSelectorInternetRow() {
  return (
    <LocationSelector.Row position="bottom">
      <FlexColumn gap="tiny">
        <LocationSelector.Row.Content>
          <LocationSelector.Row.Icon icon="internet" />
          <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
        </LocationSelector.Row.Content>
      </FlexColumn>
    </LocationSelector.Row>
  );
}
