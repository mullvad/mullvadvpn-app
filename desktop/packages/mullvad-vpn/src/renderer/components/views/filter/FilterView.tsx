import { useCallback, useMemo, useState } from 'react';
import styled from 'styled-components';

import { Ownership } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { Button } from '../../../lib/components';
import { View } from '../../../lib/components/view';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../app-navigation-header';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { FilterByProvider, OwnershipFilter } from './components';
import { useFilteredOwnershipOptions, useFilteredProviders, useProviders } from './hooks';

const StyledViewContent = styled(View.Content)`
  margin-bottom: 0;
`;

export function FilterView() {
  const history = useHistory();
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const initialProviders = useProviders();
  const [providers, setProviders] = useState<Record<string, boolean>>(initialProviders);

  // The daemon expects the value to be an empty list if all are selected.
  const formattedProviderList = useMemo(() => {
    // If all providers are selected it's represented as an empty array.
    return Object.values(providers).every((provider) => provider)
      ? []
      : Object.entries(providers)
          .filter(([, selected]) => selected)
          .map(([name]) => name);
  }, [providers]);

  const initialOwnership = useSelector((state) =>
    'normal' in state.settings.relaySettings
      ? state.settings.relaySettings.normal.ownership
      : Ownership.any,
  );
  const [ownership, setOwnership] = useState<Ownership>(initialOwnership);

  // Available providers are used to only show compatible options after activating a filter.
  const availableProviders = useFilteredProviders([], ownership);
  const availableOwnershipOptions = useFilteredOwnershipOptions(
    formattedProviderList,
    Ownership.any,
  );

  // Applies the changes by sending them to the daemon.
  const onApply = useCallback(async () => {
    await relaySettingsUpdater((settings) => {
      settings.providers = formattedProviderList;
      settings.ownership = ownership;
      return settings;
    });
    history.pop();
  }, [formattedProviderList, ownership, history, relaySettingsUpdater]);

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
                <OwnershipFilter
                  ownership={ownership}
                  availableOptions={availableOwnershipOptions}
                  setOwnership={setOwnership}
                />
                <FilterByProvider
                  providers={providers}
                  availableOptions={availableProviders}
                  setProviders={setProviders}
                />
              </View.Container>
            </NavigationScrollbars>
            <View.Container horizontalMargin="medium" padding={{ vertical: 'large' }}>
              <Button
                variant="success"
                disabled={Object.values(providers).every((provider) => !provider)}
                onClick={onApply}>
                <Button.Text>{messages.gettext('Apply')}</Button.Text>
              </Button>
            </View.Container>
          </StyledViewContent>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}
