import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import {
  BlockAdsSetting,
  BlockAdultContentSetting,
  BlockGamblingSetting,
  BlockMalwareSetting,
  BlockSocialMediaSetting,
  BlockTrackersSetting,
} from '../../../../../features/dns/components';
import { useDns } from '../../../../../features/dns/hooks';
import { AccordionProps } from '../../../../../lib/components/accordion';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { formatHtml } from '../../../../../lib/html-formatter';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsAccordion } from '../../../../settings-accordion';
import { CustomDnsEnabledFooter } from './components';

export type DnsBlockerSettingsProps = Omit<AccordionProps, 'children'> &
  Pick<ListItemProps, 'position'>;

const StyledAccordionTrigger = styled(SettingsAccordion.Trigger)`
  display: grid;
  place-items: center;
`;

export function DnsBlockerSettings({ position, ...props }: DnsBlockerSettingsProps) {
  const { dns } = useDns();
  const customDnsFeatureName = messages.pgettext('vpn-settings-view', 'Use custom DNS server');

  return (
    <div>
      <SettingsAccordion
        accordionId="dns-blocker-setting"
        anchorId="dns-blocker-setting"
        aria-label={messages.pgettext('vpn-settings-view', 'DNS content blockers')}
        disabled={dns.state === 'custom'}
        {...props}>
        <SettingsAccordion.Container>
          <SettingsAccordion.Header position={position}>
            <SettingsAccordion.HeaderItem>
              <SettingsAccordion.Title variant="bodySmallSemibold">
                {messages.pgettext('vpn-settings-view', 'DNS content blockers')}
              </SettingsAccordion.Title>
              <SettingsAccordion.HeaderActionGroup>
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
                <StyledAccordionTrigger>
                  <SettingsAccordion.Icon />
                </StyledAccordionTrigger>
              </SettingsAccordion.HeaderActionGroup>
            </SettingsAccordion.HeaderItem>
          </SettingsAccordion.Header>
          <SettingsAccordion.Content>
            <BlockAdsSetting position="middle" />
            <BlockTrackersSetting />
            <BlockMalwareSetting />
            <BlockGamblingSetting />
            <BlockAdultContentSetting />
            <BlockSocialMediaSetting />
          </SettingsAccordion.Content>
        </SettingsAccordion.Container>
      </SettingsAccordion>
      {dns.state === 'custom' && <CustomDnsEnabledFooter />}
    </div>
  );
}
