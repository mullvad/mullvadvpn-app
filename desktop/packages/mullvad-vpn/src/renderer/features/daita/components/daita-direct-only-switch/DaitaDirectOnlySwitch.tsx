import React from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import { InfoDialog } from '../../../../components/info-dialog';
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
      <InfoDialog open={confirmDialogOpen} onOpenChange={setConfirmDialogOpen}>
        <InfoDialog.Text>
          {sprintf(
            // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
            messages.pgettext(
              'wireguard-settings-view',
              'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after enabling.',
            ),
            { daita: strings.daita },
          )}
        </InfoDialog.Text>
        <InfoDialog.ButtonGroup>
          <InfoDialog.Button onClick={confirmEnableDirectOnly}>
            <InfoDialog.Button.Text>
              {
                // TRANSLATORS: A toggle that refers to the setting "Direct only".
                messages.gettext('Enable direct only')
              }
            </InfoDialog.Button.Text>
          </InfoDialog.Button>
          <InfoDialog.CloseButton>
            <InfoDialog.CloseButton.Text>
              {messages.pgettext('wireguard-settings-view', 'Cancel')}
            </InfoDialog.CloseButton.Text>
          </InfoDialog.CloseButton>
        </InfoDialog.ButtonGroup>
      </InfoDialog>
    </>
  );
}

const DaitaDirectOnlySwitchNamespace = Object.assign(DaitaDirectOnlySwitch, {
  Label: Switch.Label,
  Input: Switch.Input,
  Trigger: Switch.Trigger,
});

export { DaitaDirectOnlySwitchNamespace as DaitaDirectOnlySwitch };
