import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { Text } from '../../../../../lib/components';
import { Accordion } from '../../../../../lib/components/accordion';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useRelayListContext } from '../../RelayListContext';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { AddListForm } from '../add-list-form/AddListForm';
import { RelayLocationList } from '../relay-location-list';
import { CustomListsSectionTitle } from './components';
import { CustomListsProvider, useCustomListsContext } from './CustomListsContext';
import { useHandleOnSelectCustomList, useHasCustomLists } from './hooks';
import { useHasNoCustomListsInSearchResult } from './hooks/use-has-no-custom-lists-in-search-result';

export type LocationSelection = 'entry' | 'exit';

export type CustomListsProps = {
  locationSelection: LocationSelection;
  selectedElementRef: React.Ref<HTMLDivElement>;
};

function CustomListsImpl({ selectedElementRef }: Pick<CustomListsProps, 'selectedElementRef'>) {
  const { customLists, expandLocation, collapseLocation, onBeforeExpand } = useRelayListContext();
  const { resetHeight } = useScrollPositionContext();
  const { addFormVisible } = useCustomListsContext();
  const handleOnSelect = useHandleOnSelectCustomList();
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
            <AddListForm />
          </Accordion.Content>
        </Accordion>

        {hasCustomLists && (
          <RelayLocationList
            source={customLists}
            onExpand={expandLocation}
            onCollapse={collapseLocation}
            onWillExpand={onBeforeExpand}
            selectedElementRef={selectedElementRef}
            onSelect={handleOnSelect}
            onTransitionEnd={resetHeight}
            allowAddToCustomList={false}
          />
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

export function CustomLists({ locationSelection, ...props }: CustomListsProps) {
  return (
    <CustomListsProvider locationSelection={locationSelection}>
      <CustomListsImpl {...props} />
    </CustomListsProvider>
  );
}
