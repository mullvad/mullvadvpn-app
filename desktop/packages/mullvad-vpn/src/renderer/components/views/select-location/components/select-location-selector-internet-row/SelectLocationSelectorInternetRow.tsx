import { messages } from '../../../../../../shared/gettext';
import { useActiveFilters } from '../../../../../features/locations/hooks';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { FilterChips } from '../filter-chips';

// TODO RENAME
export function SelectLocationSelectorInternetRow() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive: showFilterChips } = useActiveFilters(locationType);

  return (
    <LocationSelector.Row position="bottom">
      <FlexColumn gap="small">
        <LocationSelector.Row.Content>
          <LocationSelector.Row.Icon icon="internet" />
          <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
        </LocationSelector.Row.Content>
        {showFilterChips && <FilterChips />}
      </FlexColumn>
    </LocationSelector.Row>
  );
}
