import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { colors, spacings } from '../../../../../../../lib/foundations';
import { AriaInput, AriaInputGroup, AriaLabel } from '../../../../../../AriaGroup';
import * as Cell from '../../../../../../cell';
import { useDns } from '../../hooks';

const StyledSectionItem = styled(Cell.Container)({
  backgroundColor: colors.blue40,
});

const IndentedValueLabel = styled(Cell.ValueLabel)({
  marginLeft: spacings.medium,
});

export function BlockAdsSetting() {
  const [dns, setBlockAds] = useDns('blockAds');

  return (
    <AriaInputGroup>
      <StyledSectionItem disabled={dns.state === 'custom'}>
        <AriaLabel>
          <IndentedValueLabel>
            {
              // TRANSLATORS: Label for settings that enables ad blocking.
              messages.pgettext('vpn-settings-view', 'Ads')
            }
          </IndentedValueLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch
            isOn={dns.state === 'default' && dns.defaultOptions.blockAds}
            onChange={setBlockAds}
          />
        </AriaInput>
      </StyledSectionItem>
    </AriaInputGroup>
  );
}
