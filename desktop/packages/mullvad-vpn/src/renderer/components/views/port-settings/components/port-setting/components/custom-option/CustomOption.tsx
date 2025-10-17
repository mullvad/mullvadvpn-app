import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { removeNonNumericCharacters } from '../../../../../../../../shared/string-helpers';
import { isInRanges } from '../../../../../../../../shared/utils';
import { useSelector } from '../../../../../../../redux/store';
import { SettingsListbox } from '../../../../../../settings-listbox';

type CustomOptionProps = {
  defaultValue?: string;
};

export function CustomOption({ defaultValue }: CustomOptionProps) {
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);
  const validateValue = React.useCallback(
    (value: number) => isInRanges(value, allowedPortRanges),
    [allowedPortRanges],
  );

  const validateStringValue = React.useCallback(
    (value: string) => {
      const numericValue = parseInt(value, 10);
      if (Number.isNaN(numericValue)) return false;
      return validateValue(numericValue);
    },
    [validateValue],
  );
  return (
    <SettingsListbox.InputOption
      defaultValue={defaultValue}
      value="custom"
      validate={validateStringValue}
      format={removeNonNumericCharacters}>
      <SettingsListbox.InputOption.Label>
        {messages.gettext('Custom')}
      </SettingsListbox.InputOption.Label>
      <SettingsListbox.InputOption.Input
        placeholder={messages.pgettext('wireguard-settings-view', 'Port')}
        maxLength={5}
        type="text"
        inputMode="numeric"
      />
    </SettingsListbox.InputOption>
  );
}
