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

export function BlockGambling() {
  const [dns, setBlockGambling] = useDns('blockGambling');

  return (
    <AriaInputGroup>
      <StyledSectionItem disabled={dns.state === 'custom'}>
        <AriaLabel>
          <IndentedValueLabel>
            {
              // TRANSLATORS: Label for settings that enables block of gamling related websites.
              messages.pgettext('vpn-settings-view', 'Gambling')
            }
          </IndentedValueLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.Switch
            isOn={dns.state === 'default' && dns.defaultOptions.blockGambling}
            onChange={setBlockGambling}
          />
        </AriaInput>
      </StyledSectionItem>
    </AriaInputGroup>
  );
}
