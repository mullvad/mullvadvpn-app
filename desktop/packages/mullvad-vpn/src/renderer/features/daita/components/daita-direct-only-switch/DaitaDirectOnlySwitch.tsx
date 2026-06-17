import React from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import { StatusDialog } from '../../../../components/status-dialog';
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

  const [confirmDialogOpen, setConfirmDialogOpen] = React.useState(false);

  const setDirectOnly = React.useCallback(
    (value: boolean) => {
      if (value) {
        setConfirmDialogOpen(true);
      } else {
        void setDaitaDirectOnly(value);
      }
    },
    [setDaitaDirectOnly],
  );

  const confirmEnableDirectOnly = React.useCallback(() => {
    void setDaitaDirectOnly(true);
    setConfirmDialogOpen(false);
  }, [setDaitaDirectOnly]);

  return (
    <>
      <Switch checked={checked} onCheckedChange={setDirectOnly} disabled={disabled} {...props}>
        {children}
      </Switch>
      <StatusDialog variant="info" open={confirmDialogOpen} onOpenChange={setConfirmDialogOpen}>
        <StatusDialog.Text>
          {sprintf(
            // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
            messages.pgettext(
              'wireguard-settings-view',
              'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after enabling.',
            ),
            { daita: strings.daita },
          )}
        </StatusDialog.Text>
        <StatusDialog.ButtonGroup>
          <StatusDialog.Button onClick={confirmEnableDirectOnly}>
            <StatusDialog.Button.Text>
              {
                // TRANSLATORS: A toggle that refers to the setting "Direct only".
                messages.gettext('Enable direct only')
              }
            </StatusDialog.Button.Text>
          </StatusDialog.Button>
          <StatusDialog.CloseButton>
            <StatusDialog.CloseButton.Text>
              {messages.pgettext('wireguard-settings-view', 'Cancel')}
            </StatusDialog.CloseButton.Text>
          </StatusDialog.CloseButton>
        </StatusDialog.ButtonGroup>
      </StatusDialog>
    </>
  );
}

const DaitaDirectOnlySwitchNamespace = Object.assign(DaitaDirectOnlySwitch, {
  Label: Switch.Label,
  Input: Switch.Input,
  Trigger: Switch.Trigger,
});

export { DaitaDirectOnlySwitchNamespace as DaitaDirectOnlySwitch };
