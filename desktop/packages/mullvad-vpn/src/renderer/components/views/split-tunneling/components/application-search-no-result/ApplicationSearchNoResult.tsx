import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { spacings } from '../../../../../lib/foundations';
import { formatHtml } from '../../../../../lib/html-formatter';
import { CellFooter, CellFooterText } from '../../../../cell';

export const StyledNoResult = styled(CellFooter)({
  display: 'flex',
  flexDirection: 'column',
  paddingTop: 0,
  marginTop: 0,
  marginBottom: spacings.large,
});

export const StyledNoResultText = styled(CellFooterText)({
  textAlign: 'center',
});

export type NoSearchResultProps = {
  searchTerm: string;
};

export function ApplicationSearchNoResult({ searchTerm }: NoSearchResultProps) {
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
