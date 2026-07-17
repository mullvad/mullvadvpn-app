import React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Button, Flex, Icon, LabelTiny } from '../../../../../lib/components';
import { spacings } from '../../../../../lib/foundations';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

const StyledFlex = styled(Flex)`
  padding: ${spacings.medium} ${spacings.small};
`;

export function NoSearchResult() {
  const { searchTerm, setSearchTerm } = useSelectLocationViewContext();

  const handleClearSearch = React.useCallback(() => {
    React.startTransition(() => {
      setSearchTerm('');
    });
  }, [setSearchTerm]);

  return (
    <StyledFlex gap="medium" flexDirection="column" alignItems="space-between" flexGrow={1}>
      <Flex
        flexGrow={1}
        flexDirection="column"
        justifyContent="center"
        aria-live="assertive"
        aria-atomic="true">
        <Flex flexDirection="column" gap="medium" alignItems="center">
          <Icon icon="search" size="big" />
          <Flex flexDirection="column">
            <LabelTiny color="whiteAlpha60" textAlign="center">
              {formatHtml(
                sprintf(
                  messages.pgettext('select-location-view', 'No result for: “%(searchTerm)s“'),
                  {
                    searchTerm,
                  },
                ),
              )}
            </LabelTiny>
            <LabelTiny color="whiteAlpha60" textAlign="center">
              {messages.gettext('Try a different search.')}
            </LabelTiny>
          </Flex>
        </Flex>
      </Flex>
      <Flex alignItems="flex-end" flexShrink={1}>
        <Button onClick={handleClearSearch}>
          <Button.Text>{messages.pgettext('select-location-view', 'Clear search')}</Button.Text>
        </Button>
      </Flex>
    </StyledFlex>
  );
}
