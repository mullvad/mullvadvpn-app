import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { CombinedLocationList, type CombinedLocationListProps } from '../combined-location-list';

export function LocationList<T>(props: CombinedLocationListProps<T>) {
  const { searchTerm } = useSelectLocationViewContext();

  if (
    searchTerm !== '' &&
    !props.relayLocations.some((country) => country.visible) &&
    (props.specialLocations === undefined || props.specialLocations.length === 0)
  ) {
    return null;
  } else {
    return (
      <FlexColumn gap="tiny">
        <SectionTitle>
          <SectionTitle.Title>
            {messages.pgettext('select-location-view', 'All locations')}
          </SectionTitle.Title>
          <SectionTitle.Divider />
        </SectionTitle>
        <CombinedLocationList {...props} />
      </FlexColumn>
    );
  }
}
