import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { formatHtml } from '../../../../../lib/html-formatter';
import {
  StyledSelectionUnavailable,
  StyledSelectionUnavailableText,
} from '../../SelectLocationStyles';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export function NoSearchResult() {
  const { searchTerm } = useSelectLocationViewContext();
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
