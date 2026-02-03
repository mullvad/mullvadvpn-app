import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useCustomLists } from '../../../../../features/location/hooks';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useLocationRowContext } from '../location-row/LocationRowContext';

type DeleteConfirmDialogProps = Omit<DialogProps, 'children'>;

export function DeleteConfirmDialog({ open, onOpenChange }: DeleteConfirmDialogProps) {
  const { deleteCustomList } = useCustomLists();
  const { source } = useLocationRowContext();

  const handleConfirm = React.useCallback(async () => {
    if (source.location.customList) {
      try {
        await deleteCustomList(source.location.customList);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to delete custom list ${source.location.customList}: ${error.message}`);
      }
    }
  }, [deleteCustomList, source.location.customList]);

  const handleCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  if (!('list' in source)) {
    return;
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="alert-circle" color="red" />
            <Dialog.Text>
              {formatHtml(
                sprintf(
                  messages.pgettext(
                    'select-location-view',
                    'Do you want to delete the list <b>%(list)s</b>?',
                  ),
                  { list: source.list.name },
                ),
              )}
            </Dialog.Text>
            <Dialog.ButtonGroup>
              <Dialog.Button key="save" variant="destructive" onClick={handleConfirm}>
                <Dialog.Button.Text>{messages.gettext('Delete list')}</Dialog.Button.Text>
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
