import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { Button } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { OwnershipFilter, ProviderFilter } from './components';
import { FilterViewContextProvider, useFilterViewContext } from './FilterViewContext';
import { useHandleApplyFilter } from './hooks';

const StyledViewContent = styled(View.Content)`
  margin-bottom: 0;
`;

function FilterViewImpl() {
  const history = useHistory();
  const { availableProviders, locationType, selectedProviders } = useFilterViewContext();
  const handleApply = useHandleApplyFilter();

  const noSelectedProviders = availableProviders.every(
    (provider) => !selectedProviders.includes(provider),
  );

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
              <Button color="success" disabled={noSelectedProviders} onClick={handleApply}>
                <Button.Text>
                  {noSelectedProviders
                    ? messages.gettext('No matching servers found')
                    : messages.gettext('Apply')}
                </Button.Text>
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
