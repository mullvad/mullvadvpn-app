import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useCustomListLocationsContext } from '../../CustomListLocationsContext';
import { useLocationsContext } from '../../LocationsContext';
import {
  StyledSelectionUnavailable,
  StyledSelectionUnavailableText,
} from '../../SelectLocationStyles';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export interface NoSearchResultProps {
  specialLocationsLength: number;
}
export function NoSearchResult(props: NoSearchResultProps) {
  const { searchedLocations } = useLocationsContext();
  const { customListLocations } = useCustomListLocationsContext();
  const { searchTerm } = useSelectLocationViewContext();

  if (
    searchTerm === '' ||
    searchedLocations.length > 0 ||
    customListLocations.length > 0 ||
    props.specialLocationsLength > 0
  ) {
    return null;
  }

  return (
    <StyledSelectionUnavailable>
      <StyledSelectionUnavailableText>
        {formatHtml(
          sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), {
            searchTerm,
          }),
        )}
      </StyledSelectionUnavailableText>
      <StyledSelectionUnavailableText>
        {messages.gettext('Try a different search.')}
      </StyledSelectionUnavailableText>
    </StyledSelectionUnavailable>
  );
}
