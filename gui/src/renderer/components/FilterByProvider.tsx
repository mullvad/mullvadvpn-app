import { useCallback, useMemo, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
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

enum Selection {
  all,
  some,
  none,
}

export default function FilterByProvider() {
  const history = useHistory();
  const { updateRelaySettings } = useAppContext();

  const serverList = useSelector((state) =>
    state.settings.relayLocations.concat(
      state.settings.bridgeState === 'on' ? state.settings.bridgeLocations : [],
    ),
  );
  const providerConstraint = useSelector((state) => {
    if ('normal' in state.settings.relaySettings) {
      return state.settings.relaySettings.normal.providers;
    } else {
      return [];
    }
  });

  const [providers, setProviders] = useState(() => {
    const providers = serverList.flatMap((country) =>
      country.cities.flatMap((city) => city.relays.map((relay) => relay.provider)),
    );
    const uniqueProviders = removeDuplicates(providers).sort((a, b) => a.localeCompare(b));

    return Object.fromEntries(
      uniqueProviders.map((provider) => [
        provider,
        providerConstraint.length === 0 || providerConstraint.includes(provider),
      ]),
    );
  });

  const selectionStatus = useMemo(() => {
    if (Object.values(providers).every((value) => value)) {
      return Selection.all;
    } else if (Object.values(providers).every((value) => !value)) {
      return Selection.none;
    } else {
      return Selection.some;
    }
  }, [providers]);

  const onCheck = useCallback((provider: string) => {
    setProviders((providers) => ({ ...providers, [provider]: !providers[provider] }));
  }, []);

  const toggleAll = useCallback(() => {
    setProviders((providers) =>
      Object.fromEntries(
        Object.keys(providers).map((provider) => [provider, selectionStatus !== Selection.all]),
      ),
    );
  }, [selectionStatus]);

  const onApply = useCallback(async () => {
    const selectedProviders =
      selectionStatus === Selection.all
        ? []
        : Object.entries(providers)
            .filter(([, selected]) => selected)
            .map(([provider]) => provider);

    await updateRelaySettings({ normal: { providers: selectedProviders } });

    history.pop();
  }, [providers, history, updateRelaySettings, selectionStatus]);

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
                    messages.pgettext('filter-by-provider-nav', 'Filter by provider')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>
            <StyledNavigationScrollbars>
              <ProviderRow
                provider={messages.pgettext('filter-by-provider-view', 'All providers')}
                bold
                checked={selectionStatus === Selection.all}
                onCheck={toggleAll}
              />
              {Object.entries(providers).map(([provider, checked]) => (
                <ProviderRow
                  key={provider}
                  provider={provider}
                  checked={checked}
                  onCheck={onCheck}
                />
              ))}
            </StyledNavigationScrollbars>
            <StyledFooter>
              <AppButton.GreenButton
                disabled={selectionStatus === Selection.none}
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

interface IProviderRowProps extends IStyledRowTitleProps {
  provider: string;
  checked: boolean;
  onCheck: (provider: string) => void;
}

function ProviderRow(props: IProviderRowProps) {
  const onCheck = useCallback(() => props.onCheck(props.provider), [props.onCheck, props.provider]);

  return (
    <StyledRow onClick={onCheck}>
      <StyledCheckbox role="checkbox" aria-label={props.provider} aria-checked={props.checked}>
        {props.checked && <ImageView source="icon-tick" width={18} tintColor={colors.green} />}
      </StyledCheckbox>
      <StyledRowTitle aria-hidden bold={props.bold}>
        {props.provider}
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
