import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Text } from '../../../../../lib/components';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { useCustomListLocationsContext } from '../../CustomListLocationsContext';
import { AddCustomListForm } from '../add-custom-list-form/AddCustomListForm';
import { CustomListLocationListItem } from '../custom-list-location-list-item';
import { CustomListsSectionTitle } from './components';
import {
  CustomListLocationListProvider,
  useCustomListListContext,
} from './CustomListLocationListContext';
import {
  useHandleSelectCustomList,
  useHasCustomLists,
  useHasNoCustomListsInSearchResult,
} from './hooks';

export type LocationSelection = 'entry' | 'exit';

export type CustomListListProps = {
  locationSelection: LocationSelection;
  selectedElementRef: React.Ref<HTMLDivElement>;
};

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
`;

const StyledAnimatedListItem = styled(AnimatedList.Item)`
  // Add spacing to the last child
  & > :last-child {
    margin-bottom: ${spacings.small};
  }
`;

function CustomListLocationListImpl({
  selectedElementRef,
}: Pick<CustomListListProps, 'selectedElementRef'>) {
  const { customListLocations } = useCustomListLocationsContext();
  const { addFormVisible, addingForm } = useCustomListListContext();
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
        <StyledAnimatedList>
          {addFormVisible && (
            <AnimatedList.Item>
              <AddCustomListForm />
            </AnimatedList.Item>
          )}
          {customListLocations.map((customList) => {
            return (
              <StyledAnimatedListItem key={Object.values(customList.details).join('-')}>
                <CustomListLocationListItem
                  customList={customList}
                  level={0}
                  selectedElementRef={selectedElementRef}
                  onSelect={handleSelectCustomList}
                />
              </StyledAnimatedListItem>
            );
          })}
        </StyledAnimatedList>

        {!hasCustomLists && !addFormVisible && !addingForm && (
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
