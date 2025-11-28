import React from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../../../components/Modal';
import { Button } from '../../../../lib/components';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useBoolean } from '../../../../lib/utility-hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../hooks';

export type DaitaDirectOnlySwitchProps = SwitchProps;

function DaitaDirectOnlySwitch({ children, ...props }: DaitaDirectOnlySwitchProps) {
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly, setDaitaDirectOnly } = useDaitaDirectOnly();

  const relaySettings = useNormalRelaySettings();
  const unavailable = relaySettings === undefined;
  const disabled = !daitaEnabled || unavailable;
  const checked = daitaDirectOnly && !unavailable;

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const setDirectOnly = React.useCallback(
    (value: boolean) => {
      if (value) {
        showConfirmationDialog();
      } else {
        void setDaitaDirectOnly(value);
      }
    },
    [setDaitaDirectOnly, showConfirmationDialog],
  );

  const confirmEnableDirectOnly = React.useCallback(() => {
    void setDaitaDirectOnly(true);
    hideConfirmationDialog();
  }, [hideConfirmationDialog, setDaitaDirectOnly]);

  return (
    <>
      <Switch checked={checked} onCheckedChange={setDirectOnly} disabled={disabled} {...props}>
        {children}
      </Switch>
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.caution}
        gridButtons={[
          <Button key="cancel" onClick={hideConfirmationDialog}>
            <Button.Text>{messages.pgettext('wireguard-settings-view', 'Cancel')}</Button.Text>
          </Button>,
          <Button key="confirm" onClick={confirmEnableDirectOnly}>
            <Button.Text>
              {
                // TRANSLATORS: A toggle that refers to the setting "Direct only".
                messages.gettext('Enable direct only')
              }
            </Button.Text>
          </Button>,
        ]}
        close={hideConfirmationDialog}>
        <ModalMessage>
          {sprintf(
            // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
            messages.pgettext(
              'wireguard-settings-view',
              'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after enabling.',
            ),
            { daita: strings.daita },
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}

const DaitaDirectOnlySwitchNamespace = Object.assign(DaitaDirectOnlySwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { DaitaDirectOnlySwitchNamespace as DaitaDirectOnlySwitch };
