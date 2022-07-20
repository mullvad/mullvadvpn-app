import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../config.json';
import { IpVersion } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { useAppContext } from '../context';
import { createWireguardRelayUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import Selector, { ISelectorItem } from './cell/Selector';
import { InfoIcon } from './InfoButton';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;
const WIREUGARD_UDP_PORTS = [51820, 53];

type OptionalPort = number | undefined;
type OptionalIpVersion = IpVersion | undefined;

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export const StyledCellIcon = styled(Cell.UntintedIcon)({
  marginRight: '8px',
});

export const StyledInfoIcon = styled(InfoIcon)({
  marginRight: '16px',
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export default function WireguardSettings() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>
                  {sprintf(
                    // TRANSLATORS: Title label in navigation bar
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                    messages.pgettext('wireguard-settings-nav', '%(wireguard)s settings'),
                    { wireguard: strings.wireguard },
                  )}
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {sprintf(
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                    messages.pgettext('wireguard-settings-view', '%(wireguard)s settings'),
                    { wireguard: strings.wireguard },
                  )}
                </HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <PortSelector />
                </Cell.Group>

                <Cell.Group>
                  <MultihopSetting />
                </Cell.Group>

                <Cell.Group>
                  <IpVersionSetting />
                </Cell.Group>

                <Cell.Group>
                  <MtuSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function PortSelector() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const { updateRelaySettings } = useAppContext();

  const wireguardPortItems = useMemo(() => {
    const automaticPort: ISelectorItem<OptionalPort> = {
      label: messages.gettext('Automatic'),
      value: undefined,
    };

    return [automaticPort].concat(WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem));
  }, []);

  const port = useMemo(() => {
    const port = 'normal' in relaySettings ? relaySettings.normal.wireguard.port : undefined;
    return port === 'any' ? undefined : port;
  }, [relaySettings]);

  const setWireguardPort = useCallback(
    async (port?: number) => {
      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => {
          if (port) {
            wireguard.port.exact(port);
          } else {
            wireguard.port.any();
          }
        })
        .build();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettings],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the WireGuard port selector.
          title={messages.pgettext('wireguard-settings-view', 'Port')}
          values={wireguardPortItems}
          value={port}
          onSelect={setWireguardPort}
        />
      </StyledSelectorContainer>
      <Cell.Footer>
        <AriaDescription>
          <Cell.FooterText>
            {
              // TRANSLATORS: The hint displayed below the WireGuard port selector.
              messages.pgettext(
                'wireguard-settings-view',
                'The automatic setting will randomly choose from a wide range of ports.',
              )
            }
          </Cell.FooterText>
        </AriaDescription>
      </Cell.Footer>
    </AriaInputGroup>
  );
}

function MultihopSetting() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const { updateRelaySettings } = useAppContext();

  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const setMultihopImpl = useCallback(
    async (enabled: boolean) => {
      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => wireguard.useMultihop(enabled))
        .build();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
    [relaySettings, updateRelaySettings],
  );

  const setMultihop = useCallback(
    async (newValue: boolean) => {
      if (newValue) {
        showConfirmationDialog();
      } else {
        await setMultihopImpl(false);
      }
    },
    [setMultihopImpl],
  );

  const confirmMultihop = useCallback(async () => {
    await setMultihopImpl(true);
    hideConfirmationDialog();
  }, [setMultihopImpl]);

  return (
    <>
      <AriaInputGroup>
        <Cell.Container>
          <AriaLabel>
            <Cell.InputLabel>
              {
                // TRANSLATORS: The label next to the multihop settings toggle.
                messages.pgettext('vpn-settings-view', 'Enable multihop')
              }
            </Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch isOn={multihop} onChange={setMultihop} />
          </AriaInput>
        </Cell.Container>
        <Cell.Footer>
          <AriaDescription>
            <Cell.FooterText>
              {sprintf(
                // TRANSLATORS: Description for multihop settings toggle.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                messages.pgettext(
                  'vpn-settings-view',
                  'Increases anonymity by routing your traffic into one %(wireguard)s server and out another, making it harder to trace.',
                ),
                { wireguard: strings.wireguard },
              )}
            </Cell.FooterText>
          </AriaDescription>
        </Cell.Footer>
      </AriaInputGroup>
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.info}
        message={
          // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
          messages.gettext('This setting increases latency. Use only if needed.')
        }
        buttons={[
          <AppButton.RedButton key="confirm" onClick={confirmMultihop}>
            {messages.gettext('Enable anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={hideConfirmationDialog}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={hideConfirmationDialog}
      />
    </>
  );
}

function IpVersionSetting() {
  const { updateRelaySettings } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const ipVersion = useMemo(() => {
    const ipVersion =
      'normal' in relaySettings ? relaySettings.normal.wireguard.ipVersion : undefined;
    return ipVersion === 'any' ? undefined : ipVersion;
  }, [relaySettings]);

  const ipVersionItems: ISelectorItem<OptionalIpVersion>[] = useMemo(
    () => [
      {
        label: messages.gettext('Automatic'),
        value: undefined,
      },
      {
        label: messages.gettext('IPv4'),
        value: 'ipv4',
      },
      {
        label: messages.gettext('IPv6'),
        value: 'ipv6',
      },
    ],
    [],
  );

  const setIpVersion = useCallback(
    async (ipVersion?: IpVersion) => {
      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => {
          if (ipVersion) {
            wireguard.ipVersion.exact(ipVersion);
          } else {
            wireguard.ipVersion.any();
          }
        })
        .build();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettings, updateRelaySettings],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the WireGuard IP version selector.
          title={messages.pgettext('wireguard-settings-view', 'IP version')}
          values={ipVersionItems}
          value={ipVersion}
          onSelect={setIpVersion}
        />
      </StyledSelectorContainer>
      <Cell.Footer>
        <AriaDescription>
          <Cell.FooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the WireGuard IP version selector.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
              messages.pgettext(
                'wireguard-settings-view',
                'This allows access to %(wireguard)s for devices that only support IPv6.',
              ),
              { wireguard: strings.wireguard },
            )}
          </Cell.FooterText>
        </AriaDescription>
      </Cell.Footer>
    </AriaInputGroup>
  );
}

function removeNonNumericCharacters(value: string) {
  return value.replace(/[^0-9]/g, '');
}

function mtuIsValid(mtu: string): boolean {
  const parsedMtu = mtu ? parseInt(mtu) : undefined;
  return (
    parsedMtu === undefined ||
    (parsedMtu >= MIN_WIREGUARD_MTU_VALUE && parsedMtu <= MAX_WIREGUARD_MTU_VALUE)
  );
}

function MtuSetting() {
  const { setWireguardMtu: setWireguardMtuImpl } = useAppContext();
  const mtu = useSelector((state) => state.settings.wireguard.mtu);

  const setMtu = useCallback(
    async (mtu?: number) => {
      try {
        await setWireguardMtuImpl(mtu);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mtu value', error.message);
      }
    },
    [setWireguardMtuImpl],
  );

  const onSubmit = useCallback(
    async (value: string) => {
      const parsedValue = value === '' ? undefined : parseInt(value, 10);
      if (mtuIsValid(value)) {
        await setMtu(parsedValue);
      }
    },
    [setMtu],
  );

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('wireguard-settings-view', 'MTU')}</Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.AutoSizingTextInput
            value={mtu ? mtu.toString() : ''}
            inputMode={'numeric'}
            maxLength={4}
            placeholder={messages.gettext('Default')}
            onSubmitValue={onSubmit}
            validateValue={mtuIsValid}
            submitOnBlur={true}
            modifyValue={removeNonNumericCharacters}
          />
        </AriaInput>
      </Cell.Container>
      <Cell.Footer>
        <AriaDescription>
          <Cell.FooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the WireGuard MTU input field.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
              // TRANSLATORS: %(max)d - the maximum possible wireguard mtu value
              // TRANSLATORS: %(min)d - the minimum possible wireguard mtu value
              messages.pgettext(
                'wireguard-settings-view',
                'Set %(wireguard)s MTU value. Valid range: %(min)d - %(max)d.',
              ),
              {
                wireguard: strings.wireguard,
                min: MIN_WIREGUARD_MTU_VALUE,
                max: MAX_WIREGUARD_MTU_VALUE,
              },
            )}
          </Cell.FooterText>
        </AriaDescription>
      </Cell.Footer>
    </AriaInputGroup>
  );
}
