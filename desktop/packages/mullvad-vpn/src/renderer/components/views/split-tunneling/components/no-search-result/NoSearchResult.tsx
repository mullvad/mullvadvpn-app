import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { formatHtml } from '../../../../../lib/html-formatter';
import { StyledNoResult, StyledNoResultText } from './NoSearchResultStyles';

export type NoSearchResultProps = {
  searchTerm: string;
};

export function NoSearchResult({ searchTerm }: NoSearchResultProps) {
  return (
    <StyledNoResult>
      <StyledNoResultText>
        {formatHtml(
          sprintf(messages.gettext('No result for <b>%(searchTerm)s</b>.'), { searchTerm }),
        )}
      </StyledNoResultText>
      <StyledNoResultText>{messages.gettext('Try a different search.')}</StyledNoResultText>
    </StyledNoResult>
  );
}
