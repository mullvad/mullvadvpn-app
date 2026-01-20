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
import { useHandleApply } from './hooks';

const StyledViewContent = styled(View.Content)`
  margin-bottom: 0;
`;

function FilterViewImpl() {
  const history = useHistory();
  const { availableProviders, selectedProviders } = useFilterViewContext();
  const handleApply = useHandleApply();

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
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('filter-nav', 'Filter')
              }
              titleVisible
            />
            <NavigationScrollbars>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="small">
                <OwnershipFilter />
                <ProviderFilter />
              </View.Container>
            </NavigationScrollbars>
            <View.Container horizontalMargin="medium" padding={{ vertical: 'large' }}>
              <Button variant="success" disabled={noSelectedProviders} onClick={handleApply}>
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
