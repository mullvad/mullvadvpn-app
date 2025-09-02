import { useCallback } from 'react';
import React from 'react';
import { sprintf } from 'sprintf-js';

import { Constraint, ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useAppContext } from '../../../../../context';
import { useScrollToListItem } from '../../../../../hooks';
import { Text } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useSelector } from '../../../../../redux/store';
import { DefaultListboxOption } from '../../../../default-listbox-option';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SplitListboxOption } from '../../../../split-listbox-option';

export function formatPortForSubLabel(port: Constraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port.only}`;
}

export function ObfuscationSettings() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const id = 'obfuscation-setting';
  const ref = React.useRef<HTMLDivElement>(null);
  const scrollToAnchor = useScrollToListItem(ref, id);

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('wireguard-settings-view', 'Port: %(port)s');

  const obfuscationType = obfuscationSettings.selectedObfuscation;

  const selectObfuscationType = useCallback(
    async (value: ObfuscationType) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        selectedObfuscation: value,
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  return (
    <Listbox
      onValueChange={selectObfuscationType}
      value={obfuscationType}
      animation={scrollToAnchor?.animation}>
      <Listbox.Item ref={ref}>
        <Listbox.Content>
          <Listbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard obfuscation selector.
              messages.pgettext('wireguard-settings-view', 'Obfuscation')
            }
          </Listbox.Label>
          <InfoButton>
            <ModalMessage>
              {
                // TRANSLATORS: Describes what WireGuard obfuscation does, how it works and when
                // TRANSLATORS: it would be useful to enable it.
                messages.pgettext(
                  'wireguard-settings-view',
                  'Obfuscation hides the WireGuard traffic inside another protocol. It can be used to help circumvent censorship and other types of filtering, where a plain WireGuard connection would be blocked.',
                )
              }
            </ModalMessage>
          </InfoButton>
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={ObfuscationType.auto} data-testid="automatic-obfuscation">
          {messages.gettext('Automatic')}
        </DefaultListboxOption>
        <SplitListboxOption value={ObfuscationType.shadowsocks}>
          <SplitListboxOption.Item>
            <FlexColumn>
              <Listbox.Option.Label>
                {messages.pgettext('wireguard-settings-view', 'Shadowsocks')}
              </Listbox.Option.Label>
              <Text variant="labelTiny" color="whiteAlpha60">
                {sprintf(subLabelTemplate, {
                  port: formatPortForSubLabel(obfuscationSettings.shadowsocksSettings.port),
                })}
              </Text>
            </FlexColumn>
          </SplitListboxOption.Item>
          <SplitListboxOption.NavigateButton
            to={RoutePath.shadowsocks}
            aria-description={messages.pgettext('accessibility', 'Shadowsocks settings')}
          />
        </SplitListboxOption>
        <SplitListboxOption value={ObfuscationType.udp2tcp}>
          <SplitListboxOption.Item>
            <FlexColumn>
              <Listbox.Option.Label>
                {messages.pgettext('wireguard-settings-view', 'UDP-over-TCP')}
              </Listbox.Option.Label>
              <Text variant="labelTiny" color="whiteAlpha60">
                {sprintf(subLabelTemplate, {
                  port: formatPortForSubLabel(obfuscationSettings.udp2tcpSettings.port),
                })}
              </Text>
            </FlexColumn>
          </SplitListboxOption.Item>
          <SplitListboxOption.NavigateButton
            to={RoutePath.udpOverTcp}
            aria-description={messages.pgettext('accessibility', 'UDP-over-TCP settings')}
          />
        </SplitListboxOption>
        <DefaultListboxOption value={ObfuscationType.quic}>
          {messages.pgettext('wireguard-settings-view', 'QUIC')}
        </DefaultListboxOption>
        <DefaultListboxOption value={ObfuscationType.off}>
          {messages.gettext('Off')}
        </DefaultListboxOption>
      </Listbox.Options>
    </Listbox>
  );
}
