import { useCallback, useMemo, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { Ownership } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utilityHooks';
import { IReduxState, useSelector } from '../redux/store';
import Accordion from './Accordion';
import * as AppButton from './AppButton';
import { AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import Selector from './cell/Selector';
import { normalText } from './common-styles';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Container, Layout } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';

const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  backgroundColor: colors.darkBlue,
  flex: 1,
});

const StyledFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  padding: '18px 22px 22px',
});

export default function Filter() {
  const history = useHistory();
  const { updateRelaySettings } = useAppContext();

  const initialProviders = useSelector(providersSelector);
  const [providers, setProviders] = useState<Record<string, boolean>>(initialProviders);

  const initialOwnership = useSelector((state) =>
    'normal' in state.settings.relaySettings
      ? state.settings.relaySettings.normal.ownership
      : Ownership.any,
  );
  const [ownership, setOwnership] = useState<Ownership>(initialOwnership);

  const onApply = useCallback(async () => {
    // If all providers are selected it's represented as an empty array.
    const selectedProviders = Object.values(providers).every((provider) => provider)
      ? []
      : Object.entries(providers)
          .filter(([, selected]) => selected)
          .map(([name]) => name);

    await updateRelaySettings({ normal: { providers: selectedProviders, ownership } });
    history.pop();
  }, [providers, ownership, history, updateRelaySettings]);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar alwaysDisplayBarTitle={true}>
              <NavigationItems>
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('filter-nav', 'Filter')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>
            <StyledNavigationScrollbars>
              <FilterByOwnership ownership={ownership} setOwnership={setOwnership} />
              <FilterByProvider providers={providers} setProviders={setProviders} />
            </StyledNavigationScrollbars>
            <StyledFooter>
              <AppButton.GreenButton
                disabled={Object.values(providers).every((provider) => !provider)}
                onClick={onApply}>
                {messages.gettext('Apply')}
              </AppButton.GreenButton>
            </StyledFooter>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
    </BackAction>
  );
}

function providersSelector(state: IReduxState): Record<string, boolean> {
  const providerConstraint =
    'normal' in state.settings.relaySettings ? state.settings.relaySettings.normal.providers : [];

  const relays = state.settings.relayLocations.concat(
    state.settings.bridgeState === 'on' ? state.settings.bridgeLocations : [],
  );
  const providers = relays.flatMap((country) =>
    country.cities.flatMap((city) => city.relays.map((relay) => relay.provider)),
  );
  const uniqueProviders = removeDuplicates(providers).sort((a, b) => a.localeCompare(b));

  // Empty containt array means that all providers are selected. No selection isn't possible.
  return Object.fromEntries(
    uniqueProviders.map((provider) => [
      provider,
      providerConstraint.length === 0 || providerConstraint.includes(provider),
    ]),
  );
}

const StyledSelector = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

interface IFilterByOwnershipProps {
  ownership: Ownership;
  setOwnership: (ownership: Ownership) => void;
}

function FilterByOwnership(props: IFilterByOwnershipProps) {
  const [expanded, , , toggleExpanded] = useBoolean(false);

  const values = useMemo(
    () => [
      {
        label: messages.gettext('Any'),
        value: Ownership.any,
      },
      {
        label: messages.pgettext('filter-view', 'Mullvad owned only'),
        value: Ownership.mullvadOwned,
      },
      {
        label: messages.pgettext('filter-view', 'Rented only'),
        value: Ownership.rented,
      },
    ],
    [],
  );

  return (
    <AriaInputGroup>
      <Cell.CellButton onClick={toggleExpanded}>
        <AriaLabel>
          <Cell.Label>{messages.pgettext('filter-view', 'Ownership')}</Cell.Label>
        </AriaLabel>
        <ImageView
          tintColor={colors.white80}
          source={expanded ? 'icon-chevron-up' : 'icon-chevron-down'}
          height={24}
        />
      </Cell.CellButton>

      <Accordion expanded={expanded}>
        <StyledSelector values={values} value={props.ownership} onSelect={props.setOwnership} />
      </Accordion>
    </AriaInputGroup>
  );
}

interface IFilterByProviderProps {
  providers: Record<string, boolean>;
  setProviders: (providers: (previous: Record<string, boolean>) => Record<string, boolean>) => void;
}

function FilterByProvider(props: IFilterByProviderProps) {
  const [expanded, , , toggleExpanded] = useBoolean(false);

  const onToggle = useCallback(
    (provider: string) =>
      props.setProviders((providers) => ({ ...providers, [provider]: !providers[provider] })),
    [props.setProviders],
  );

  const toggleAll = useCallback(() => {
    props.setProviders((providers) => {
      const shouldSelect = !Object.values(providers).every((value) => value);
      return Object.fromEntries(Object.keys(providers).map((provider) => [provider, shouldSelect]));
    });
  }, []);

  return (
    <>
      <Cell.CellButton onClick={toggleExpanded}>
        <Cell.Label>{messages.pgettext('filter-view', 'Providers')}</Cell.Label>
        <ImageView
          tintColor={colors.white80}
          source={expanded ? 'icon-chevron-up' : 'icon-chevron-down'}
          height={24}
        />
      </Cell.CellButton>
      <Accordion expanded={expanded}>
        <CheckboxRow
          label={messages.pgettext('filter-view', 'All providers')}
          bold
          checked={Object.values(props.providers).every((value) => value)}
          onChange={toggleAll}
        />
        {Object.entries(props.providers).map(([provider, checked]) => (
          <CheckboxRow key={provider} label={provider} checked={checked} onChange={onToggle} />
        ))}
      </Accordion>
    </>
  );
}

interface IStyledRowTitleProps {
  bold?: boolean;
}

const StyledRow = styled.div({
  display: 'flex',
  height: '44px',
  alignItems: 'center',
  padding: '0 22px',
  marginBottom: '1px',
  backgroundColor: colors.blue,
});

const StyledCheckbox = styled.div({
  width: '24px',
  height: '24px',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  backgroundColor: colors.white,
  borderRadius: '4px',
});

const StyledRowTitle = styled.label(normalText, (props: IStyledRowTitleProps) => ({
  fontWeight: props.bold ? 600 : 400,
  color: colors.white,
  marginLeft: '22px',
}));

interface ICheckboxRowProps extends IStyledRowTitleProps {
  label: string;
  checked: boolean;
  onChange: (provider: string) => void;
}

function CheckboxRow(props: ICheckboxRowProps) {
  const onToggle = useCallback(() => props.onChange(props.label), [props.onChange, props.label]);

  return (
    <StyledRow onClick={onToggle}>
      <StyledCheckbox role="checkbox" aria-label={props.label} aria-checked={props.checked}>
        {props.checked && <ImageView source="icon-tick" width={18} tintColor={colors.green} />}
      </StyledCheckbox>
      <StyledRowTitle aria-hidden bold={props.bold}>
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
