import { useCallback, useMemo, useState } from 'react';
import styled from 'styled-components';

import { Ownership } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { Icon } from '../lib/components';
import { useRelaySettingsUpdater } from '../lib/constraint-updater';
import {
  EndpointType,
  filterLocations,
  filterLocationsByEndPointType,
} from '../lib/filter-locations';
import { Colors } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { useNormalRelaySettings, useTunnelProtocol } from '../lib/relay-settings-hooks';
import { useBoolean } from '../lib/utility-hooks';
import { IRelayLocationCountryRedux } from '../redux/settings/reducers';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import { AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import Selector from './cell/Selector';
import { normalText } from './common-styles';
import { BackAction } from './KeyboardNavigation';
import { Footer, Layout, SettingsContainer } from './Layout';
import { NavigationContainer } from './NavigationContainer';
import { NavigationScrollbars } from './NavigationScrollbars';

const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  backgroundColor: Colors.darkBlue,
  flex: 1,
});

export default function Filter() {
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
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('filter-nav', 'Filter')
              }
              titleVisible
            />
            <StyledNavigationScrollbars>
              <FilterByOwnership
                ownership={ownership}
                availableOptions={availableOwnershipOptions}
                setOwnership={setOwnership}
              />
              <FilterByProvider
                providers={providers}
                availableOptions={availableProviders}
                setProviders={setProviders}
              />
            </StyledNavigationScrollbars>
            <Footer>
              <AppButton.GreenButton
                disabled={Object.values(providers).every((provider) => !provider)}
                onClick={onApply}>
                {messages.gettext('Apply')}
              </AppButton.GreenButton>
            </Footer>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

// Returns only the ownership options that are compatible with the other filters
function useFilteredOwnershipOptions(providers: string[], ownership: Ownership): Ownership[] {
  const relaySettings = useNormalRelaySettings();
  const tunnelProtocol = useTunnelProtocol();
  const bridgeState = useSelector((state) => state.settings.bridgeState);
  const locations = useSelector((state) => state.settings.relayLocations);

  const endpointType = bridgeState === 'on' ? EndpointType.any : EndpointType.exit;

  const availableOwnershipOptions = useMemo(() => {
    const relayListForEndpointType = filterLocationsByEndPointType(
      locations,
      endpointType,
      tunnelProtocol,
      relaySettings,
    );
    const relaylistForFilters = filterLocations(relayListForEndpointType, ownership, providers);

    const filteredRelayOwnership = relaylistForFilters.flatMap((country) =>
      country.cities.flatMap((city) => city.relays.map((relay) => relay.owned)),
    );

    const ownershipOptions = [Ownership.any];
    if (filteredRelayOwnership.includes(true)) {
      ownershipOptions.push(Ownership.mullvadOwned);
    }
    if (filteredRelayOwnership.includes(false)) {
      ownershipOptions.push(Ownership.rented);
    }

    return ownershipOptions;
  }, [locations, endpointType, tunnelProtocol, relaySettings, ownership, providers]);

  return availableOwnershipOptions;
}

// Returns only the providers that are compatible with the other filters
export function useFilteredProviders(providers: string[], ownership: Ownership): string[] {
  const relaySettings = useNormalRelaySettings();
  const tunnelProtocol = useTunnelProtocol();
  const bridgeState = useSelector((state) => state.settings.bridgeState);
  const locations = useSelector((state) => state.settings.relayLocations);

  const endpointType = bridgeState === 'on' ? EndpointType.any : EndpointType.exit;

  const availableProviders = useMemo(() => {
    const relayListForEndpointType = filterLocationsByEndPointType(
      locations,
      endpointType,
      tunnelProtocol,
      relaySettings,
    );
    const relaylistForFilters = filterLocations(relayListForEndpointType, ownership, providers);
    return providersFromRelays(relaylistForFilters);
  }, [endpointType, locations, ownership, providers, relaySettings, tunnelProtocol]);

  return availableProviders;
}

// Returns all available providers in the provided relay list.
function providersFromRelays(relays: IRelayLocationCountryRedux[]) {
  const providers = relays.flatMap((country) =>
    country.cities.flatMap((city) => city.relays.map((relay) => relay.provider)),
  );
  return removeDuplicates(providers).sort((a, b) => a.localeCompare(b));
}

function useProviders(): Record<string, boolean> {
  const tunnelProtocol = useTunnelProtocol();
  const relaySettings = useNormalRelaySettings();
  const relayLocations = useSelector((state) => state.settings.relayLocations);
  const bridgeState = useSelector((state) => state.settings.bridgeState);
  const providerConstraint = relaySettings?.providers ?? [];

  const endpointType =
    tunnelProtocol === 'openvpn' && bridgeState === 'on' ? EndpointType.any : EndpointType.exit;
  const relays = filterLocationsByEndPointType(
    relayLocations,
    endpointType,
    tunnelProtocol,
    relaySettings,
  );
  const providers = providersFromRelays(relays);

  // Empty containt array means that all providers are selected. No selection isn't possible.
  return Object.fromEntries(
    providers.map((provider) => [
      provider,
      providerConstraint.length === 0 || providerConstraint.includes(provider),
    ]),
  );
}

const StyledSelector = styled(Selector)({
  marginBottom: 0,
}) as typeof Selector;

interface IFilterByOwnershipProps {
  ownership: Ownership;
  availableOptions: Ownership[];
  setOwnership: (ownership: Ownership) => void;
}

function FilterByOwnership(props: IFilterByOwnershipProps) {
  const [expanded, , , toggleExpanded] = useBoolean(false);

  const values = useMemo(
    () =>
      [
        {
          label: messages.pgettext('filter-view', 'Mullvad owned only'),
          value: Ownership.mullvadOwned,
        },
        {
          label: messages.pgettext('filter-view', 'Rented only'),
          value: Ownership.rented,
        },
      ].filter((option) => props.availableOptions.includes(option.value)),
    [props.availableOptions],
  );

  return (
    <AriaInputGroup>
      <Cell.CellButton onClick={toggleExpanded}>
        <AriaLabel>
          <Cell.Label>{messages.pgettext('filter-view', 'Ownership')}</Cell.Label>
        </AriaLabel>
        <Icon color={Colors.white80} icon={expanded ? 'chevron-up' : 'chevron-down'} />
      </Cell.CellButton>

      <Accordion expanded={expanded}>
        <StyledSelector
          items={values}
          value={props.ownership}
          onSelect={props.setOwnership}
          automaticLabel={messages.gettext('Any')}
          automaticValue={Ownership.any}
        />
      </Accordion>
    </AriaInputGroup>
  );
}

interface IFilterByProviderProps {
  providers: Record<string, boolean>;
  availableOptions: string[];
  setProviders: (providers: (previous: Record<string, boolean>) => Record<string, boolean>) => void;
}

function FilterByProvider(props: IFilterByProviderProps) {
  const { setProviders } = props;

  const [expanded, , , toggleExpanded] = useBoolean(false);

  const onToggle = useCallback(
    (provider: string) =>
      setProviders((providers) => {
        const newProviders = { ...providers, [provider]: !providers[provider] };
        return props.availableOptions.every((provider) => newProviders[provider])
          ? toggleAllProviders(providers, true)
          : newProviders;
      }),
    [props.availableOptions, setProviders],
  );

  const toggleAll = useCallback(() => {
    setProviders((providers) => toggleAllProviders(providers));
  }, [setProviders]);

  return (
    <>
      <Cell.CellButton onClick={toggleExpanded}>
        <Cell.Label>{messages.pgettext('filter-view', 'Providers')}</Cell.Label>
        <Icon color={Colors.white80} icon={expanded ? 'chevron-up' : 'chevron-down'} />
      </Cell.CellButton>
      <Accordion expanded={expanded}>
        <CheckboxRow
          label={messages.pgettext('filter-view', 'All providers')}
          $bold
          checked={Object.values(props.providers).every((value) => value)}
          onChange={toggleAll}
        />
        {Object.entries(props.providers)
          .filter(([provider]) => props.availableOptions.includes(provider))
          .map(([provider, checked]) => (
            <CheckboxRow key={provider} label={provider} checked={checked} onChange={onToggle} />
          ))}
      </Accordion>
    </>
  );
}

function toggleAllProviders(providers: Record<string, boolean>, value?: boolean) {
  const shouldSelect = value ?? !Object.values(providers).every((value) => value);
  return Object.fromEntries(Object.keys(providers).map((provider) => [provider, shouldSelect]));
}

interface IStyledRowTitleProps {
  $bold?: boolean;
}

const StyledCheckbox = styled.div({
  width: '24px',
  height: '24px',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  backgroundColor: Colors.white,
  borderRadius: '4px',
});

const StyledRow = styled(Cell.Row)({
  backgroundColor: Colors.blue40,
  '&&:hover': {
    backgroundColor: Colors.blue80,
  },
});

const StyledRowTitle = styled.label<IStyledRowTitleProps>(normalText, (props) => ({
  fontWeight: props.$bold ? 600 : 400,
  color: Colors.white,
  marginLeft: '22px',
}));

interface ICheckboxRowProps extends IStyledRowTitleProps {
  label: string;
  checked: boolean;
  onChange: (provider: string) => void;
}

function CheckboxRow(props: ICheckboxRowProps) {
  const { onChange } = props;

  const onToggle = useCallback(() => onChange(props.label), [onChange, props.label]);

  return (
    <StyledRow onClick={onToggle}>
      <StyledCheckbox role="checkbox" aria-label={props.label} aria-checked={props.checked}>
        {props.checked && <Icon icon="checkmark" color={Colors.green} />}
      </StyledCheckbox>
      <StyledRowTitle aria-hidden $bold={props.$bold}>
        {props.label}
      </StyledRowTitle>
    </StyledRow>
  );
}

function removeDuplicates(list: string[]): string[] {
  return list.reduce(
    (result, current) => (result.includes(current) ? result : [...result, current]),
    [] as string[],
  );
}
