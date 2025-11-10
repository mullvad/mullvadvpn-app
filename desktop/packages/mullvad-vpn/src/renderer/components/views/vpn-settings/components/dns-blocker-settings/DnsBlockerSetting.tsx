import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import {
  BlockAdsSetting,
  BlockAdultContentSetting,
  BlockGamblingSetting,
  BlockMalwareSetting,
  BlockSocialMediaSetting,
  BlockTrackersSetting,
} from '../../../../../features/dns/components';
import { FlexRow } from '../../../../../lib/components/flex-row';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsAccordion } from '../../../../settings-accordion';
import { CustomDnsEnabledFooter } from './components';

export function DnsBlockerSettings() {
  const dns = useSelector((state) => state.settings.dns);
  const customDnsFeatureName = messages.pgettext('vpn-settings-view', 'Use custom DNS server');

  return (
    <>
      <SettingsAccordion
        accordionId="dns-blocker-setting"
        anchorId="dns-blocker-setting"
        aria-label={messages.pgettext('vpn-settings-view', 'DNS content blockers')}
        disabled={dns.state === 'custom'}>
        <SettingsAccordion.Header>
          <SettingsAccordion.Title>
            {messages.pgettext('vpn-settings-view', 'DNS content blockers')}
          </SettingsAccordion.Title>
          <FlexRow $gap="medium">
            <InfoButton>
              <ModalMessage>
                {messages.pgettext(
                  'vpn-settings-view',
                  'When this feature is enabled it stops the device from contacting certain domains or websites known for distributing ads, malware, trackers and more.',
                )}
              </ModalMessage>
              <ModalMessage>
                {messages.pgettext(
                  'vpn-settings-view',
                  'This might cause issues on certain websites, services, and apps.',
                )}
              </ModalMessage>
              <ModalMessage>
                {formatHtml(
                  sprintf(
                    messages.pgettext(
                      'vpn-settings-view',
                      'Attention: this setting cannot be used in combination with <b>%(customDnsFeatureName)s</b>',
                    ),
                    { customDnsFeatureName },
                  ),
                )}
              </ModalMessage>
            </InfoButton>
            <SettingsAccordion.Trigger>
              <SettingsAccordion.Icon />
            </SettingsAccordion.Trigger>
          </FlexRow>
        </SettingsAccordion.Header>
        <SettingsAccordion.Content>
          <BlockAdsSetting />
          <BlockTrackersSetting />
          <BlockMalwareSetting />
          <BlockGamblingSetting />
          <BlockAdultContentSetting />
          <BlockSocialMediaSetting />
        </SettingsAccordion.Content>
      </SettingsAccordion>
      {dns.state === 'custom' && <CustomDnsEnabledFooter />}
    </>
  );
}
