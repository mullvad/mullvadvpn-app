import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function EnableIpv6Setting() {
  const enableIpv6 = useSelector((state) => state.settings.enableIpv6);
  const { setEnableIpv6: setEnableIpv6Impl } = useAppContext();

  const setEnableIpv6 = useCallback(
    async (enableIpv6: boolean) => {
      try {
        await setEnableIpv6Impl(enableIpv6);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update enable IPv6', error.message);
      }
    },
    [setEnableIpv6Impl],
  );

  return (
    <SettingsToggleListItem
      checked={enableIpv6}
      onCheckedChange={setEnableIpv6}
      description={
        // TRANSLATORS: Description of in-tunnel IPv6 setting.
        messages.pgettext('vpn-settings-view', 'Enable to allow IPv6 traffic through the tunnel.')
      }>
      <SettingsToggleListItem.Label>
        {
          // TRANSLATORS: Title of in-tunnel IPv6 setting.
          messages.pgettext('vpn-settings-view', 'In-tunnel IPv6')
        }
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Group>
        <InfoButton>
          <ModalMessage>
            {messages.pgettext(
              'vpn-settings-view',
              'When this feature is enabled, IPv6 can be used alongside IPv4 in the VPN tunnel to communicate with internet services.',
            )}
          </ModalMessage>
          <ModalMessage>
            {messages.pgettext(
              'vpn-settings-view',
              'IPv4 is always enabled and the majority of websites and applications use this protocol. We do not recommend enabling IPv6 unless you know you need it.',
            )}
          </ModalMessage>
        </InfoButton>
        <SettingsToggleListItem.Switch />
      </SettingsToggleListItem.Group>
    </SettingsToggleListItem>
  );
}
