import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../features/location/hooks';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { formatHtml } from '../../../../../lib/html-formatter';
import { type CustomListLocation } from '../../select-location-types';
import { useCustomListLocationListItemContext } from '../custom-list-location-list-item/CustomListLocationListItemContext';

type ConfirmDeleteCustomListDialogProps = Omit<DialogProps, 'children'> & {
  customList: CustomListLocation;
};

export function ConfirmDeleteCustomListDialog({
  customList,
  open,
  onOpenChange,
}: ConfirmDeleteCustomListDialogProps) {
  const { deleteCustomList } = useCustomLists();
  const { setLoading } = useCustomListLocationListItemContext();

  const handleConfirm = React.useCallback(async () => {
    onOpenChange?.(false);
    setLoading(true);
    const success = await deleteCustomList(customList.details.customList);

    // Only set loading to false if failed to keep disabled state while animating out
    if (!success) {
      setLoading(false);
    }
  }, [customList.details.customList, deleteCustomList, onOpenChange, setLoading]);

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
