import React from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import { Button } from '../../../../lib/components';
import { Dialog } from '../../../../lib/components/dialog';
import { Switch, SwitchProps } from '../../../../lib/components/switch';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { useDaitaDirectOnly, useDaitaEnabled } from '../../hooks';

export type DaitaDirectOnlySwitchProps = SwitchProps;

function DaitaDirectOnlySwitch({ children, ...props }: DaitaDirectOnlySwitchProps) {
  const { daitaEnabled } = useDaitaEnabled();
  const { daitaDirectOnly, setDaitaDirectOnly } = useDaitaDirectOnly();

  const relaySettings = useNormalRelaySettings();
  const unavailable = relaySettings === undefined;
  const disabled = !daitaEnabled || unavailable;
  const checked = daitaDirectOnly && !unavailable;

  const [confirmDialogVisible, setConfirmDialogVisible] = React.useState(false);

  const hideConfirmationDialog = React.useCallback(() => {
    setConfirmDialogVisible(false);
  }, [setConfirmDialogVisible]);

  const setDirectOnly = React.useCallback(
    (value: boolean) => {
      if (value) {
        setConfirmDialogVisible(true);
      } else {
        void setDaitaDirectOnly(value);
      }
    },
    [setDaitaDirectOnly, setConfirmDialogVisible],
  );

  const confirmEnableDirectOnly = React.useCallback(() => {
    void setDaitaDirectOnly(true);
    hideConfirmationDialog();
  }, [hideConfirmationDialog, setDaitaDirectOnly]);

  console.log('confirmDialogVisible', confirmDialogVisible);

  return (
    <>
      <Switch checked={checked} onCheckedChange={setDirectOnly} disabled={disabled} {...props}>
        {children}
      </Switch>
      <Dialog open={confirmDialogVisible} onOpenChange={setConfirmDialogVisible}>
        <Dialog.Container>
          <Dialog.Icon icon="info-circle" />
          <Dialog.Text>
            {sprintf(
              // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
              messages.pgettext(
                'wireguard-settings-view',
                'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after enabling.',
              ),
              { daita: strings.daita },
            )}
          </Dialog.Text>
          <Dialog.ButtonGroup>
            <Button key="confirm" onClick={confirmEnableDirectOnly}>
              <Button.Text>
                {
                  // TRANSLATORS: A toggle that refers to the setting "Direct only".
                  messages.gettext('Enable direct only')
                }
              </Button.Text>
            </Button>
            <Dialog.Button key="cancel" onClick={hideConfirmationDialog}>
              <Button.Text>{messages.pgettext('wireguard-settings-view', 'Cancel')}</Button.Text>
            </Dialog.Button>
          </Dialog.ButtonGroup>
        </Dialog.Container>
      </Dialog>
    </>
  );
}

const DaitaDirectOnlySwitchNamespace = Object.assign(DaitaDirectOnlySwitch, {
  Label: Switch.Label,
  Thumb: Switch.Thumb,
  Trigger: Switch.Trigger,
});

export { DaitaDirectOnlySwitchNamespace as DaitaDirectOnlySwitch };
