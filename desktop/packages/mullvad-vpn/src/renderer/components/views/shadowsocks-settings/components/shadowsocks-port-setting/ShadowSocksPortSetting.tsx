import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInputGroup } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import { SelectorItem, SelectorWithCustomItem } from '../../../../cell/Selector';

const PORTS: Array<SelectorItem<number>> = [];
const ALLOWED_RANGE = [1, 65535];

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function ShadowsocksPortSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const port =
    obfuscationSettings.shadowsocksSettings.port === 'any'
      ? null
      : obfuscationSettings.shadowsocksSettings.port.only;

  const setShadowsocksPort = useCallback(
    async (port: number | null) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        shadowsocksSettings: {
          ...obfuscationSettings.shadowsocksSettings,
          port: wrapConstraint(port),
        },
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  const parseValue = useCallback((port: string) => parseInt(port), []);

  const validateValue = useCallback(
    (value: number) => value >= ALLOWED_RANGE[0] && value <= ALLOWED_RANGE[1],
    [],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <SelectorWithCustomItem
          // TRANSLATORS: The title for the WireGuard port selector.
          title={messages.pgettext('wireguard-settings-view', 'Port')}
          items={PORTS}
          value={port}
          onSelect={setShadowsocksPort}
          inputPlaceholder={messages.pgettext('wireguard-settings-view', 'Port')}
          automaticValue={null}
          parseValue={parseValue}
          modifyValue={removeNonNumericCharacters}
          validateValue={validateValue}
          maxLength={`${ALLOWED_RANGE[1]}`.length}
        />
      </StyledSelectorContainer>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: Text describing the valid port range for a port selector.
              messages.pgettext('wireguard-settings-view', 'Valid range: %(min)s - %(max)s'),
              { min: ALLOWED_RANGE[0], max: ALLOWED_RANGE[1] },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}
