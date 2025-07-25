import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { colors, spacings } from '../../../../../../../lib/foundations';
import { formatHtml } from '../../../../../../../lib/html-formatter';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from '../../../../../../AriaGroup';
import * as Cell from '../../../../../../cell';
import { useDns } from '../../hooks';

const StyledSectionItem = styled(Cell.Container)({
  backgroundColor: colors.blue40,
});

const IndentedValueLabel = styled(Cell.ValueLabel)({
  marginLeft: spacings.medium,
});

export function BlockSocialMedia() {
  const [dns, setBlockSocialMedia] = useDns('blockSocialMedia');

  return (
    <AriaInputGroup>
      <StyledSectionItem disabled={dns.state === 'custom'}>
        <AriaLabel>
          <IndentedValueLabel>
            {
              // TRANSLATORS: Label for settings that enables block of social media.
              messages.pgettext('vpn-settings-view', 'Social media')
            }
          </IndentedValueLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch
            isOn={dns.state === 'default' && dns.defaultOptions.blockSocialMedia}
            onChange={setBlockSocialMedia}
          />
        </AriaInput>
      </StyledSectionItem>
      {dns.state === 'custom' && <CustomDnsEnabledFooter />}
    </AriaInputGroup>
  );
}

function CustomDnsEnabledFooter() {
  const customDnsFeatureName = messages.pgettext('vpn-settings-view', 'Use custom DNS server');

  // TRANSLATORS: This is displayed when the custom DNS setting is turned on which makes the block
  // TRANSLATORS: ads/trackers settings disabled. The text enclosed in "<b></b>" will appear bold.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(customDnsFeatureName)s - The name displayed next to the custom DNS toggle.
  const blockingDisabledText = messages.pgettext(
    'vpn-settings-view',
    'Disable <b>%(customDnsFeatureName)s</b> below to activate these settings.',
  );

  return (
    <Cell.CellFooter>
      <AriaDescription>
        <Cell.CellFooterText>
          {formatHtml(sprintf(blockingDisabledText, { customDnsFeatureName }))}
        </Cell.CellFooterText>
      </AriaDescription>
    </Cell.CellFooter>
  );
}
