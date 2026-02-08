import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { RelayLocationList, type RelayLocationListProps } from '../relay-location-list';

export function LocationList({ locations, onSelect, ...props }: RelayLocationListProps) {
  const { searchTerm } = useSelectLocationViewContext();

  if (searchTerm !== '' && !locations.some((location) => location.visible)) {
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
        <RelayLocationList locations={locations} onSelect={onSelect} {...props} />
      </FlexColumn>
    );
  }
}
