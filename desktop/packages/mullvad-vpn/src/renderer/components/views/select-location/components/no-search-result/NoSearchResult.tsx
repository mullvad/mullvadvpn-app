import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useRelayListContext } from '../../RelayListContext';
import {
  StyledSelectionUnavailable,
  StyledSelectionUnavailableText,
} from '../../SelectLocationStyles';
import { useSelectLocationContext } from '../../SelectLocationView';

export interface NoSearchResultProps {
  specialLocationsLength: number;
}
export function NoSearchResult(props: NoSearchResultProps) {
  const { relayList, customLists } = useRelayListContext();
  const { searchTerm } = useSelectLocationContext();

  if (
    searchTerm === '' ||
    relayList.some((country) => country.visible) ||
    customLists.some((list) => list.visible) ||
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
