import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { TextField } from '../../../../../lib/components/text-field';
import {
  AddCustomListDialogProvider,
  useAddCustomListDialogContext,
} from './AddCustomListDialogContext';
import {
  useHandleClickCreateCustomList,
  useHandleCustomListNameChange,
  useHandleSubmitAddCustomList,
} from './hooks';

export type AddCustomListDialogProps = Omit<DialogProps, 'children'>;

function AddCustomListDialogImpl(props: AddCustomListDialogProps) {
  const descriptionId = React.useId();

  const {
    open,
    onOpenChange,
    formRef,
    inputRef,
    form: {
      error,
      customListTextField: { value, invalid, dirty, invalidReason, reset },
    },
  } = useAddCustomListDialogContext();

  const handleOnValueChange = useHandleCustomListNameChange();
  const handleSubmit = useHandleSubmitAddCustomList();
  const handleClick = useHandleClickCreateCustomList();

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
                invalid={dirty && (error || invalid)}>
                <Dialog.TextGroup gap="small" flexGrow="1">
                  <TextField.Label color="whiteAlpha60">
                    {
                      // TRANSLATORS: Label for the input where the user can enter the name of a new custom list.
                      messages.pgettext('select-location-view', 'Create custom list')
                    }
                  </TextField.Label>
                  <FlexColumn gap="small">
                    <TextField.Input
                      ref={inputRef}
                      width="medium"
                      maxLength={30}
                      autoFocus
                      aria-describedby={descriptionId}
                      aria-errormessage={invalidReason ? descriptionId : undefined}
                    />
                    <Dialog.Text id={descriptionId} role="status">
                      {invalidReason
                        ? invalidReason
                        : // TRANSLATORS: Helper text under input for creating a custom list.
                          messages.pgettext(
                            'select-location-view',
                            'Enter a name for the custom list',
                          )}
                    </Dialog.Text>
                  </FlexColumn>
                </Dialog.TextGroup>
              </TextField>
            </form>
            <Dialog.ButtonGroup>
              <Dialog.Button key="save" disabled={error || invalid} onClick={handleClick}>
                <Dialog.Button.Text>{messages.gettext('Create')}</Dialog.Button.Text>
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

export function AddCustomListDialog({ open, onOpenChange, ...props }: AddCustomListDialogProps) {
  return (
    <AddCustomListDialogProvider open={open} onOpenChange={onOpenChange}>
      <AddCustomListDialogImpl {...props} />
    </AddCustomListDialogProvider>
  );
}
