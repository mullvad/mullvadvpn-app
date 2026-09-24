import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { useProviders } from '../../../features/locations/hooks';
import { Button } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { OwnershipFilter, ProviderFilter } from './components';
import { FilterViewContextProvider, useFilterViewContext } from './FilterViewContext';
import { useFilteredLocationsMatchCount, useHandleApplyFilter } from './hooks';

const StyledViewContent = styled(View.Content)`
  margin-bottom: 0;
`;

function FilterViewImpl() {
  const history = useHistory();
  const { locationType, selectedProviders } = useFilterViewContext();
  const handleApply = useHandleApplyFilter();

  const { exitProviders, entryProviders } = useProviders();
  const activeProviders = locationType === 'entry' ? entryProviders : exitProviders;

  const noSelectedProviders = activeProviders.every(
    (provider) => !selectedProviders.includes(provider),
  );

  const { isLoading, isFetching, matchingServers } = useFilteredLocationsMatchCount();
  const showNoMatchingServers = matchingServers === 0 || selectedProviders.length === 0;
  const disabled = showNoMatchingServers || isLoading || isFetching;

  const getLabel = () => {
    if (isLoading || isFetching) {
      return messages.gettext('...');
    }

    if (showNoMatchingServers) {
      return messages.gettext('No matching servers found');
    }

    return sprintf(messages.gettext('Apply %(count)s'), {
      count: matchingServers,
    });
  };

  const label = getLabel();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={history.pop}>
        <NavigationContainer>
          <StyledViewContent>
            <AppNavigationHeader
              title={
                locationType === 'entry'
                  ? // This line is here to prevent the following one to be moved up here by prettier
                    // TRANSLATORS: Title label in navigation bar for entry location filters
                    messages.pgettext('filter-nav', 'Entry filter')
                  : // This line is here to prevent the following one to be moved up here by prettier
                    // TRANSLATORS: Title label in navigation bar for exit location filters
                    messages.pgettext('filter-nav', 'Exit filter')
              }
              titleVisible
            />
            <NavigationScrollbars>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="small">
                <OwnershipFilter defaultOpen />
                <ProviderFilter defaultOpen />
              </View.Container>
            </NavigationScrollbars>
            <View.Container horizontalMargin="medium" padding={{ vertical: 'large' }}>
              <Button color="success" disabled={disabled} onClick={handleApply}>
                <Button.Text>{label}</Button.Text>
              </Button>
            </View.Container>
          </StyledViewContent>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

export function FilterView() {
  return (
    <FilterViewContextProvider>
      <FilterViewImpl />
    </FilterViewContextProvider>
  );
}
