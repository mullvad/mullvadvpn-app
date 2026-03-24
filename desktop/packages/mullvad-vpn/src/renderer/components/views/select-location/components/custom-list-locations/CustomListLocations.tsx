import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Container, Text } from '../../../../../lib/components';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useHasCustomLists } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { getLocationListItemMapProps } from '../../utils';
import { CustomListLocation } from '../custom-list-location';
import { CustomListsSectionTitle } from './components';
import {
  CustomListLocationsProvider,
  useCustomListLocationsContext,
} from './CustomListLocationsContext';

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
`;

function CustomListLocationsImpl() {
  const { addingCustomList } = useCustomListLocationsContext();
  const { customListLocations } = useSelectLocationViewContext();

  const hasCustomLists = useHasCustomLists();
  const showAddCustomListText = !hasCustomLists && !addingCustomList;
  const showAddLocationToCustomListText = hasCustomLists;

  return (
    <FlexColumn gap="tiny">
      <CustomListsSectionTitle />
      <FlexColumn>
        <StyledAnimatedList>
          {customListLocations.map((customList) => {
            const { key } = getLocationListItemMapProps(customList, undefined);
            return (
              <AnimatedList.Item key={key}>
                <CustomListLocation customList={customList} />
              </AnimatedList.Item>
            );
          })}
        </StyledAnimatedList>

        {showAddCustomListText && (
          <Text variant="labelTiny" color="whiteAlpha60">
            {
              // TRANSLATORS: Message shown when the user has no custom lists.
              // TRANSLATORS: Instructs the user how to create a custom list.
              messages.pgettext(
                'select-location-view',
                'Add a custom list by clicking the “+” icon',
              )
            }
          </Text>
        )}
        {showAddLocationToCustomListText && (
          <Container horizontalMargin="medium">
            <Text variant="labelTiny" color="whiteAlpha60">
              {
                // TRANSLATORS: Message shown in the custom list section when the user has at least one custom list.
                // TRANSLATORS: Instructs the user how to add locations to the custom list.
                messages.pgettext(
                  'select-location-view',
                  'Click “+“ on a location to add it to a list',
                )
              }
            </Text>
          </Container>
        )}
      </FlexColumn>
    </FlexColumn>
  );
}

export function CustomListLocations() {
  return (
    <CustomListLocationsProvider>
      <CustomListLocationsImpl />
    </CustomListLocationsProvider>
  );
}
