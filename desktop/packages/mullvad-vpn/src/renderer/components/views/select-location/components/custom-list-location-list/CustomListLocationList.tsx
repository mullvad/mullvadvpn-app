import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Container, Text } from '../../../../../lib/components';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { useCustomListLocationsContext } from '../../CustomListLocationsContext';
import { useHasCustomLists } from '../../hooks';
import { AddCustomListDialog } from '../add-custom-list-dialog';
import { CustomListLocationListItem } from '../custom-list-location-list-item';
import { CustomListsSectionTitle } from './components';
import {
  CustomListLocationListProvider,
  useCustomListListContext,
} from './CustomListLocationListContext';

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
`;

const StyledAnimatedListItem = styled(AnimatedList.Item)`
  // Add spacing to the last child
  & > :last-child {
    margin-bottom: ${spacings.tiny};
  }
`;

function CustomListLocationListImpl() {
  const { customListLocations } = useCustomListLocationsContext();
  const { addingCustomList, addCustomListDialogOpen, setAddCustomListDialogOpen } =
    useCustomListListContext();
  const hasCustomLists = useHasCustomLists();

  return (
    <FlexColumn gap="tiny">
      <CustomListsSectionTitle />
      <AddCustomListDialog
        open={addCustomListDialogOpen}
        onOpenChange={setAddCustomListDialogOpen}
      />

      <FlexColumn>
        <StyledAnimatedList>
          {customListLocations.map((customList) => {
            return (
              <StyledAnimatedListItem key={Object.values(customList.details).join('-')}>
                <CustomListLocationListItem customList={customList} level={0} />
              </StyledAnimatedListItem>
            );
          })}
        </StyledAnimatedList>

        {!hasCustomLists && !addingCustomList && (
          <Text variant="labelTiny" color="whiteAlpha60">
            {messages.pgettext(
              'select-location-view',
              'Add a custom list by clicking the “+” icon ',
            )}
          </Text>
        )}
        {hasCustomLists && (
          <Container horizontalMargin="small">
            <Text variant="labelTiny" color="whiteAlpha60">
              {messages.pgettext(
                'select-location-view',
                'Add locations by clicking on “+” when hovering over a location.',
              )}
            </Text>
          </Container>
        )}
      </FlexColumn>
    </FlexColumn>
  );
}

export function CustomListLocationList() {
  return (
    <CustomListLocationListProvider>
      <CustomListLocationListImpl />
    </CustomListLocationListProvider>
  );
}
