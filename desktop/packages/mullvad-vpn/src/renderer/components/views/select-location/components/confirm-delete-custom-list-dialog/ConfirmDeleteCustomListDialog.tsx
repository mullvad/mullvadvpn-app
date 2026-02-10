import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useCustomLists } from '../../../../../features/location/hooks';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { formatHtml } from '../../../../../lib/html-formatter';
import { type CustomListLocation } from '../../select-location-types';

type ConfirmDeleteCustomListDialogProps = Omit<DialogProps, 'children'> & {
  customList: CustomListLocation;
};

export function ConfirmDeleteCustomListDialog({
  customList,
  open,
  onOpenChange,
}: ConfirmDeleteCustomListDialogProps) {
  const { deleteCustomList } = useCustomLists();

  const handleConfirm = React.useCallback(async () => {
    try {
      await deleteCustomList(customList.details.customList);
    } catch (e) {
      const error = e as Error;
      log.error(`Failed to delete custom list ${customList.details.customList}: ${error.message}`);
    }
  }, [customList.details.customList, deleteCustomList]);

  const handleCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

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
                    'Do you want to delete the list %(list)s?',
                  ),
                  { list: customList.label },
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
