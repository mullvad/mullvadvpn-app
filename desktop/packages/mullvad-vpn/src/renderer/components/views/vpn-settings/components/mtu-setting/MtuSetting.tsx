import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import { useTextField } from '../../../../../lib/components/text-field';
import { useSelector } from '../../../../../redux/store';
import { SettingsListItem } from '../../../../settings-list-item';

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

  const inputRef = React.useRef<HTMLInputElement>(null);
  const labelId = React.useId();
  const descriptionId = React.useId();

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

  const { value, handleChange, invalid, dirty, blur, reset } = useTextField({
    inputRef,
    defaultValue: mtu ? mtu.toString() : '',
    format: removeNonNumericCharacters,
    validate: mtuIsValid,
  });

  const handleBlur = React.useCallback(async () => {
    if (!invalid && dirty) {
      await onSubmit(value);
    }
    if (invalid) {
      reset();
    }
  }, [dirty, invalid, onSubmit, reset, value]);

  const handleSubmit = React.useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (!invalid) {
        await onSubmit(value);
        blur();
      }
    },
    [blur, invalid, onSubmit, value],
  );

  return (
    <SettingsListItem anchorId="mtu-setting" aria-labelledby={labelId}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <SettingsListItem.Label id={labelId}>
            {
              // TRANSLATORS: The title for the WireGuard MTU setting. MTU stands for Maximum
              // TRANSLATORS: Transmission Unit and controls the maximum size of packets sent over
              // TRANSLATORS: the VPN tunnel.
              messages.pgettext('wireguard-settings-view', 'MTU')
            }
          </SettingsListItem.Label>
          <SettingsListItem.TextField invalid={invalid} onSubmit={handleSubmit}>
            <SettingsListItem.TextField.Input
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
          </SettingsListItem.TextField>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
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
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}
