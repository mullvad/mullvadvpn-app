import { messages } from '../../../../../../shared/gettext';
import * as Cell from '../../../../cell';
import { useSelectLocationContext } from '../../SelectLocationView';
import { CombinedLocationList, type CombinedLocationListProps } from '../combined-location-list';

export function LocationList<T>(props: CombinedLocationListProps<T>) {
  const { searchTerm } = useSelectLocationContext();

  if (
    searchTerm !== '' &&
    !props.relayLocations.some((country) => country.visible) &&
    (props.specialLocations === undefined || props.specialLocations.length === 0)
  ) {
    return null;
  } else {
    return (
      <>
        <Cell.Row>
          <Cell.Label>{messages.pgettext('select-location-view', 'All locations')}</Cell.Label>
        </Cell.Row>
        <CombinedLocationList {...props} />
      </>
    );
  }
}
