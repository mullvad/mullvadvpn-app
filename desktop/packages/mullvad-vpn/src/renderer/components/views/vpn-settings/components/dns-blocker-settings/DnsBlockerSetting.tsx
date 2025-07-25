import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { spacings } from '../../../../../lib/foundations';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import * as Cell from '../../../../cell';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import {
  BlockAdsSetting,
  BlockAdultContentSetting,
  BlockGamblingSetting,
  BlockMalwareSetting,
  BlockSocialMediaSetting,
  BlockTrackersSetting,
} from './components';

const StyledInfoButton = styled(InfoButton)({
  marginRight: spacings.medium,
});

const StyledTitleLabel = styled(Cell.SectionTitle)({
  flex: 1,
});

export function DnsBlockerSettings() {
  const dns = useSelector((state) => state.settings.dns);
  const customDnsFeatureName = messages.pgettext('vpn-settings-view', 'Use custom DNS server');

  const title = (
    <>
      <StyledTitleLabel as="label" disabled={dns.state === 'custom'}>
        {messages.pgettext('vpn-settings-view', 'DNS content blockers')}
      </StyledTitleLabel>
      <StyledInfoButton>
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
      </StyledInfoButton>
    </>
  );

  return (
    <Cell.ExpandableSection sectionTitle={title} expandableId="dns-blockers">
      <BlockAdsSetting />
      <BlockTrackersSetting />
      <BlockMalwareSetting />
      <BlockGamblingSetting />
      <BlockAdultContentSetting />
      <BlockSocialMediaSetting />
    </Cell.ExpandableSection>
  );
}
