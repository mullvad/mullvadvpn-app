import { messages } from '../../../../../../shared/gettext';
import { useActiveFilters } from '../../../../../features/locations/hooks';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useEntryType } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { FilterChips } from '../filter-chips';

export function SelectLocationSelectorInternetRow() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive } = useActiveFilters(locationType);
  const entryType = useEntryType();
  const showFilterChips = isAnyFilterActive && entryType !== 'entryAutomatic';

  return (
    <LocationSelector.Row position="bottom">
      <FlexColumn gap="tiny">
        <LocationSelector.Row.Content>
          <LocationSelector.Row.Icon icon="internet" />
          <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
        </LocationSelector.Row.Content>
        {showFilterChips && <FilterChips />}
      </FlexColumn>
    </LocationSelector.Row>
  );
}
