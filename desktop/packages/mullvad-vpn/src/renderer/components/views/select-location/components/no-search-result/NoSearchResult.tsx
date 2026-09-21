import React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { BodySmall, Button, Icon } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

const StyledGrid = styled.div`
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto minmax(0, 2fr);
  justify-items: center;

  flex-grow: 1;
`;

const StyledMiddleContent = styled(FlexColumn)`
  grid-row: 2;
  align-self: center;
`;

const StyledBottomContent = styled.div`
  grid-row: 3;
  align-self: end;
  width: 100%;
`;

export function NoSearchResult() {
  const { searchTerm, setSearchTerm } = useSelectLocationViewContext();

  const handleClearSearch = React.useCallback(() => {
    setSearchTerm('');
  }, [setSearchTerm]);

  return (
    <StyledGrid aria-live="assertive" aria-atomic="true">
      <StyledMiddleContent gap="medium" alignItems="center">
        <Icon icon="search" size="big" />
        <FlexColumn>
          <BodySmall color="whiteAlpha60" textAlign="center">
            {formatHtml(
              sprintf(
                messages.pgettext('select-location-view', 'No result for: “%(searchTerm)s“'),
                {
                  searchTerm,
                },
              ),
            )}
          </BodySmall>
          <BodySmall color="whiteAlpha60" textAlign="center">
            {messages.gettext('Try a different search.')}
          </BodySmall>
        </FlexColumn>
      </StyledMiddleContent>
      <StyledBottomContent>
        <Button onClick={handleClearSearch}>
          <Button.Text>{messages.pgettext('select-location-view', 'Clear search')}</Button.Text>
        </Button>
      </StyledBottomContent>
    </StyledGrid>
  );
}
