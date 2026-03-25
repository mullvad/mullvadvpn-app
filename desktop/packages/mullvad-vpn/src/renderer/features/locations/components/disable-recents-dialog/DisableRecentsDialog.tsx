import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { Dialog, type DialogProps } from '../../../../lib/components/dialog';
import { useRecents } from '../../hooks';

export type DisableRecentsDialogProps = Omit<DialogProps, 'children'>;

export function DisableRecentsDialog({ onOpenChange, ...props }: DisableRecentsDialogProps) {
  const { setEnabledRecents } = useRecents();

  const disableRecents = React.useCallback(async () => {
    await setEnabledRecents(false);
    onOpenChange?.(false);
  }, [onOpenChange, setEnabledRecents]);

  const handleCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  return (
    <Dialog onOpenChange={onOpenChange} {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="info-circle" />
            <Dialog.Text>
              {messages.pgettext('locations-feature', 'Disabling recents will also clear history.')}
            </Dialog.Text>
            <Dialog.ButtonGroup>
              <Dialog.Button variant="destructive" onClick={disableRecents}>
                <Dialog.Button.Text>{messages.gettext('Disable')}</Dialog.Button.Text>
              </Dialog.Button>
              <Dialog.Button key="cancel" onClick={handleCancel}>
                <Dialog.Button.Text>{messages.gettext('Cancel')}</Dialog.Button.Text>
              </Dialog.Button>
            </Dialog.ButtonGroup>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
