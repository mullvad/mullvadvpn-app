import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';

const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;

function mtuIsValid(mtu: string): boolean {
  const parsedMtu = mtu ? parseInt(mtu) : undefined;
  return (
    parsedMtu === undefined ||
    (parsedMtu >= MIN_WIREGUARD_MTU_VALUE && parsedMtu <= MAX_WIREGUARD_MTU_VALUE)
  );
}

export function MtuSetting() {
  const { setWireguardMtu: setWireguardMtuImpl } = useAppContext();
  const mtu = useSelector((state) => state.settings.wireguard.mtu);

  const setMtu = useCallback(
    async (mtu?: number) => {
      try {
        await setWireguardMtuImpl(mtu);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mtu value', error.message);
      }
    },
    [setWireguardMtuImpl],
  );

  const onSubmit = useCallback(
    async (value: string) => {
      const parsedValue = value === '' ? undefined : parseInt(value, 10);
      if (mtuIsValid(value)) {
        await setMtu(parsedValue);
      }
    },
    [setMtu],
  );

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('wireguard-settings-view', 'MTU')}</Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.AutoSizingTextInput
            initialValue={mtu ? mtu.toString() : ''}
            inputMode={'numeric'}
            maxLength={4}
            placeholder={messages.gettext('Default')}
            onSubmitValue={onSubmit}
            validateValue={mtuIsValid}
            submitOnBlur={true}
            modifyValue={removeNonNumericCharacters}
          />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the WireGuard MTU input field.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
              // TRANSLATORS: %(max)d - the maximum possible wireguard mtu value
              // TRANSLATORS: %(min)d - the minimum possible wireguard mtu value
              messages.pgettext(
                'wireguard-settings-view',
                'Set %(wireguard)s MTU value. Valid range: %(min)d - %(max)d.',
              ),
              {
                wireguard: strings.wireguard,
                min: MIN_WIREGUARD_MTU_VALUE,
                max: MAX_WIREGUARD_MTU_VALUE,
              },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}
