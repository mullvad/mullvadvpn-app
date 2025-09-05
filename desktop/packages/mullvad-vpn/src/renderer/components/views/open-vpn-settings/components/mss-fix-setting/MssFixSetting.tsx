import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import { useScrollToListItem } from '../../../../../hooks';
import { ListItem } from '../../../../../lib/components/list-item';
import { useTextField } from '../../../../../lib/components/text-field';
import { useSelector } from '../../../../../redux/store';

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;

export function MssFixSetting() {
  const { setOpenVpnMssfix: setOpenVpnMssfixImpl } = useAppContext();
  const mssfix = useSelector((state) => state.settings.openVpn.mssfix);

  const { ref, animation } = useScrollToListItem('mss-fix-setting');

  const inputRef = React.useRef<HTMLInputElement>(null);
  const labelId = React.useId();
  const descriptionId = React.useId();

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

  const { value, handleChange, invalid, dirty, blur, reset } = useTextField({
    inputRef,
    defaultValue: mssfix ? mssfix.toString() : '',
    format: removeNonNumericCharacters,
    validate: mssfixIsValid,
  });

  const handleBlur = React.useCallback(async () => {
    if (!invalid && dirty) {
      await onMssfixSubmit(value);
    }
    if (invalid) {
      reset();
    }
  }, [dirty, invalid, onMssfixSubmit, reset, value]);

  const handleSubmit = React.useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (!invalid) {
        await onMssfixSubmit(value);
        blur();
      }
    },
    [blur, invalid, onMssfixSubmit, value],
  );

  return (
    <ListItem animation={animation}>
      <ListItem.Item ref={ref}>
        <ListItem.Content>
          <ListItem.Label id={labelId}>
            {messages.pgettext('openvpn-settings-view', 'Mssfix')}
          </ListItem.Label>
          <ListItem.TextField invalid={invalid} onSubmit={handleSubmit}>
            <ListItem.TextField.Input
              ref={inputRef}
              value={value}
              placeholder={messages.gettext('Default')}
              inputMode="numeric"
              maxLength={4}
              aria-labelledby={labelId}
              aria-describedby={descriptionId}
              onBlur={handleBlur}
              onChange={handleChange}
            />
          </ListItem.TextField>
        </ListItem.Content>
      </ListItem.Item>
      <ListItem.Footer>
        <ListItem.Text id={descriptionId}>
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
        </ListItem.Text>
      </ListItem.Footer>
    </ListItem>
  );
}

function mssfixIsValid(mssfix: string): boolean {
  const parsedMssFix = mssfix ? parseInt(mssfix) : undefined;
  return (
    parsedMssFix === undefined ||
    (parsedMssFix >= MIN_MSSFIX_VALUE && parsedMssFix <= MAX_MSSFIX_VALUE)
  );
}
