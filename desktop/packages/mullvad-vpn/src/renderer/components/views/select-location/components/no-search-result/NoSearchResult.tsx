import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useCustomListLocationContext } from '../../CustomListLocationContext';
import { useRelayListContext } from '../../RelayListContext';
import {
  StyledSelectionUnavailable,
  StyledSelectionUnavailableText,
} from '../../SelectLocationStyles';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export interface NoSearchResultProps {
  specialLocationsLength: number;
}
export function NoSearchResult(props: NoSearchResultProps) {
  const { relayList } = useRelayListContext();
  const { customListLocations } = useCustomListLocationContext();
  const { searchTerm } = useSelectLocationViewContext();

  if (
    searchTerm === '' ||
    relayList.some((country) => country.visible) ||
    customListLocations.some((list) => list.visible) ||
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
