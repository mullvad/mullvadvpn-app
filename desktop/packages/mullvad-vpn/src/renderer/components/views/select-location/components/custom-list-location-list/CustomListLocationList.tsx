import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Text } from '../../../../../lib/components';
import { Accordion } from '../../../../../lib/components/accordion';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { useRelayListContext } from '../../RelayListContext';
import { AddCustomListForm } from '../add-custom-list-form/AddCustomListForm';
import { CustomListLocationListItem } from '../custom-list-location-list-item';
import { CustomListsSectionTitle } from './components';
import {
  CustomListLocationListProvider,
  useCustomListListContext,
} from './CustomListLocationListContext';
import { useHandleSelectCustomList, useHasCustomLists } from './hooks';
import { useHasNoCustomListsInSearchResult } from './hooks/use-has-no-custom-lists-in-search-result';

export type LocationSelection = 'entry' | 'exit';

export type CustomListListProps = {
  locationSelection: LocationSelection;
  selectedElementRef: React.Ref<HTMLDivElement>;
};

const StyledLocationContainer = styled.div`
  // If the container has children, add spacing between them
  &:not(:last-child):has(> *) {
    margin-bottom: ${spacings.small};
  }
`;

function CustomListLocationListImpl({
  selectedElementRef,
}: Pick<CustomListListProps, 'selectedElementRef'>) {
  const { customLists } = useRelayListContext();
  const { addFormVisible } = useCustomListListContext();
  const handleSelectCustomList = useHandleSelectCustomList();
  const hasNoCustomListsInSearchResult = useHasNoCustomListsInSearchResult();
  const hasCustomLists = useHasCustomLists();

  if (hasNoCustomListsInSearchResult) {
    return null;
  }

  return (
    <FlexColumn gap="tiny">
      <CustomListsSectionTitle />

      <FlexColumn>
        <Accordion expanded={addFormVisible}>
          <Accordion.Content>
            <AddCustomListForm />
          </Accordion.Content>
        </Accordion>

        {hasCustomLists && (
          <FlexColumn>
            {customLists.map((customList) => {
              return (
                <StyledLocationContainer key={Object.values(customList.details).join('-')}>
                  <CustomListLocationListItem
                    location={customList}
                    level={0}
                    selectedElementRef={selectedElementRef}
                    onSelect={handleSelectCustomList}
                  />
                </StyledLocationContainer>
              );
            })}
          </FlexColumn>
        )}

        {!hasCustomLists && !addFormVisible && (
          <Text variant="labelTiny" color="whiteAlpha60">
            {messages.pgettext(
              'select-location-view',
              'Add a custom list by clicking the “+” icon ',
            )}
          </Text>
        )}
      </FlexColumn>
    </FlexColumn>
  );
}

export function CustomListLocationList({ locationSelection, ...props }: CustomListListProps) {
  return (
    <CustomListLocationListProvider locationSelection={locationSelection}>
      <CustomListLocationListImpl {...props} />
    </CustomListLocationListProvider>
  );
}
