import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Dialog, type DialogProps } from '../../../../lib/components/dialog';
import { formatHtml } from '../../../../lib/html-formatter';
import { type CustomListLocation } from '../../../locations/types';
import { useDeleteCustomList } from '../../hooks';

type DeleteCustomListDialogProps = Omit<DialogProps, 'children'> & {
  customList: CustomListLocation;
  loading?: boolean;
  onLoadingChange?: (loading: boolean) => void;
};

export function DeleteCustomListDialog({
  customList,
  open,
  onOpenChange,
  loading,
  onLoadingChange,
}: DeleteCustomListDialogProps) {
  const deleteCustomList = useDeleteCustomList();

  const handleConfirm = React.useCallback(async () => {
    onLoadingChange?.(true);
    onOpenChange?.(false);
    const { success } = await deleteCustomList(customList.details.customList);

    // Only set loading to false if failed to keep disabled state while animating out
    if (!success) {
      onLoadingChange?.(false);
    }
  }, [customList.details.customList, deleteCustomList, onOpenChange, onLoadingChange]);

  const handleCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  // Workaround to autofocus element since autoFocus prop currently does not
  // work inside a dialog element.
  const cancelButtonRefCallback = React.useCallback((button: HTMLButtonElement | null) => {
    button?.setAttribute('autofocus', 'true');
  }, []);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="alert-circle" color="red" />
            <Dialog.Text>
              {formatHtml(
                sprintf(
                  // TRANSLATORS: Confirmation message shown when the user tries to delete a custom list.
                  // TRANSLATORS: Asks the user if they are sure they want to delete the list.
                  // TRANSLATORS: Available placeholder:
                  // TRANSLATORS: %(list)s - The name of the custom list that is about to be deleted.
                  messages.pgettext(
                    'select-location-view',
                    'Do you want to delete the list %(list)s?',
                  ),
                  { list: customList.label },
                ),
              )}
            </Dialog.Text>
            <Dialog.ButtonGroup>
              <Dialog.Button
                key="save"
                variant="destructive"
                onClick={handleConfirm}
                disabled={loading || !open}>
                <Dialog.Button.Text>{messages.gettext('Delete list')}</Dialog.Button.Text>
              </Dialog.Button>
              <Dialog.Button ref={cancelButtonRefCallback} key="cancel" onClick={handleCancel}>
                <Dialog.Button.Text>{messages.gettext('Cancel')}</Dialog.Button.Text>
              </Dialog.Button>
            </Dialog.ButtonGroup>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
