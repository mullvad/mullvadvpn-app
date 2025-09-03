import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { useScrollToListItem } from '../../../../../hooks';
import { Accordion } from '../../../../../lib/components/accordion';
import { FlexRow } from '../../../../../lib/components/flex-row';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useBoolean } from '../../../../../lib/utility-hooks';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import {
  BlockAdsSetting,
  BlockAdultContentSetting,
  BlockGamblingSetting,
  BlockMalwareSetting,
  BlockSocialMediaSetting,
  BlockTrackersSetting,
  CustomDnsEnabledFooter,
} from './components';

export function DnsBlockerSettings() {
  const dns = useSelector((state) => state.settings.dns);
  const customDnsFeatureName = messages.pgettext('vpn-settings-view', 'Use custom DNS server');
  const [expanded, , , toggleExpanded] = useBoolean();
  const { ref, animation } = useScrollToListItem('dns-blocker-setting');

  return (
    <>
      <Accordion
        ref={ref}
        expanded={expanded}
        onExpandedChange={toggleExpanded}
        disabled={dns.state === 'custom'}
        animation={animation}>
        <Accordion.Header>
          <Accordion.Title>
            {messages.pgettext('vpn-settings-view', 'DNS content blockers')}
          </Accordion.Title>
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
            <Accordion.Trigger>
              <Accordion.Icon />
            </Accordion.Trigger>
          </FlexRow>
        </Accordion.Header>
        <Accordion.Content>
          <BlockAdsSetting />
          <BlockTrackersSetting />
          <BlockMalwareSetting />
          <BlockGamblingSetting />
          <BlockAdultContentSetting />
          <BlockSocialMediaSetting />
        </Accordion.Content>
      </Accordion>
      {dns.state === 'custom' && <CustomDnsEnabledFooter />}
    </>
  );
}
