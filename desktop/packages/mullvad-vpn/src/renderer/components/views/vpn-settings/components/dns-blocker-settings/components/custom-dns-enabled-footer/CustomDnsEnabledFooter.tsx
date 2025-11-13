import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { Flex, Text } from '../../../../../../../lib/components';

export function CustomDnsEnabledFooter() {
  const customDnsFeatureName = messages.pgettext('vpn-settings-view', 'Use custom DNS server');

  // TRANSLATORS: This is displayed when the custom DNS setting is turned on which makes the block
  // TRANSLATORS: ads/trackers settings disabled. The text enclosed in "<b></b>" will appear bold.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(customDnsFeatureName)s - The name displayed next to the custom DNS toggle.
  const blockingDisabledText = messages.pgettext(
    'vpn-settings-view',
    'Disable "%(customDnsFeatureName)s" below to activate these settings.',
  );

  return (
    <Flex padding={{ top: 'tiny', horizontal: 'medium' }}>
      <Text variant="labelTinySemiBold" color="whiteAlpha60">
        {sprintf(blockingDisabledText, { customDnsFeatureName })}
      </Text>
    </Flex>
  );
}
