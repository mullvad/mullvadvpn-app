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

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;

export function MssFixSetting() {
  const { setOpenVpnMssfix: setOpenVpnMssfixImpl } = useAppContext();
  const mssfix = useSelector((state) => state.settings.openVpn.mssfix);

  const setOpenVpnMssfix = useCallback(
    async (mssfix?: number) => {
      try {
        await setOpenVpnMssfixImpl(mssfix);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mssfix value', error.message);
      }
    },
    [setOpenVpnMssfixImpl],
  );

  const onMssfixSubmit = useCallback(
    async (value: string) => {
      const parsedValue = value === '' ? undefined : parseInt(value, 10);
      if (mssfixIsValid(value)) {
        await setOpenVpnMssfix(parsedValue);
      }
    },
    [setOpenVpnMssfix],
  );

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('openvpn-settings-view', 'Mssfix')}</Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.AutoSizingTextInput
            initialValue={mssfix ? mssfix.toString() : ''}
            inputMode={'numeric'}
            maxLength={4}
            placeholder={messages.gettext('Default')}
            onSubmitValue={onMssfixSubmit}
            validateValue={mssfixIsValid}
            submitOnBlur={true}
            modifyValue={removeNonNumericCharacters}
          />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the Mssfix input field.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(openvpn)s - will be replaced with "OpenVPN"
              // TRANSLATORS: %(max)d - the maximum possible mssfix value
              // TRANSLATORS: %(min)d - the minimum possible mssfix value
              messages.pgettext(
                'openvpn-settings-view',
                'Set %(openvpn)s MSS value. Valid range: %(min)d - %(max)d.',
              ),
              {
                openvpn: strings.openvpn,
                min: MIN_MSSFIX_VALUE,
                max: MAX_MSSFIX_VALUE,
              },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function mssfixIsValid(mssfix: string): boolean {
  const parsedMssFix = mssfix ? parseInt(mssfix) : undefined;
  return (
    parsedMssFix === undefined ||
    (parsedMssFix >= MIN_MSSFIX_VALUE && parsedMssFix <= MAX_MSSFIX_VALUE)
  );
}
