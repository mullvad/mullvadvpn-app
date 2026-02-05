import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { TextField } from '../../../../../lib/components/text-field';
import { EditListDialogProvider, useEditListDialogContext } from './EditListDialogContext';
import {
  useHandleClickUpdateCustomList,
  useHandleCustomListNameChange,
  useHandleSubmitUpdateCustomList,
} from './hooks';

export type EditListProps = Omit<DialogProps, 'children'>;

function EditListDialogImpl(props: Omit<EditListProps, 'children' | 'open' | 'onOpenChange'>) {
  const {
    open,
    onOpenChange,
    formRef,
    inputRef,
    form: {
      error,
      customListTextField: { value, invalid, invalidReason, dirty, reset },
    },
  } = useEditListDialogContext();

  const handleOnValueChange = useHandleCustomListNameChange();
  const handleSubmit = useHandleSubmitUpdateCustomList();
  const handleClick = useHandleClickUpdateCustomList();

  const handleOnOpenChange = React.useCallback(
    (open: boolean) => {
      if (!open) {
        reset();
      }
      onOpenChange?.(open);
    },
    [onOpenChange, reset],
  );

  const handleCancel = React.useCallback(() => {
    handleOnOpenChange?.(false);
  }, [handleOnOpenChange]);

  return (
    <Dialog open={open} onOpenChange={handleOnOpenChange} {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <form ref={formRef} onSubmit={handleSubmit}>
              <TextField
                value={value}
                onValueChange={handleOnValueChange}
                invalid={dirty && invalid}>
                <Dialog.TextGroup gap="small" flexGrow="1">
                  <TextField.Label color="whiteAlpha60">
                    {messages.pgettext('select-location-view', 'Edit list name')}
                  </TextField.Label>
                  <FlexColumn gap="small">
                    <TextField.Input ref={inputRef} maxLength={30} autoFocus></TextField.Input>
                    <Dialog.Text>
                      {dirty && invalid && invalidReason
                        ? invalidReason
                        : messages.pgettext(
                            'select-location-view',
                            'Enter a new name for the custom list',
                          )}
                    </Dialog.Text>
                  </FlexColumn>
                </Dialog.TextGroup>
              </TextField>
            </form>
            <Dialog.ButtonGroup>
              <Dialog.Button key="save" disabled={error || invalid} onClick={handleClick}>
                <Dialog.Button.Text>{messages.gettext('Save')}</Dialog.Button.Text>
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

export function EditListDialog({ open, onOpenChange, ...prop }: EditListProps) {
  return (
    <EditListDialogProvider open={open} onOpenChange={onOpenChange}>
      <EditListDialogImpl {...prop} />
    </EditListDialogProvider>
  );
}
