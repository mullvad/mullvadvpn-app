import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { Dialog, type DialogProps } from '../../../../lib/components/dialog';
import { FlexColumn } from '../../../../lib/components/flex-column';
import { TextField } from '../../../../lib/components/text-field';
import type { GeographicalLocation } from '../../../locations/types';
import {
  CreateCustomListDialogProvider,
  useCreateCustomListDialogContext,
} from './CreateCustomListDialogContext';
import {
  useHandleClickCreateCustomList,
  useHandleCustomListNameChange,
  useHandleSubmitAddCustomList,
} from './hooks';

export type CreateCustomListDialogProps = Omit<DialogProps, 'children'> & {
  location?: GeographicalLocation;
  loading?: boolean;
  onLoadingChange?: (loading: boolean) => void;
};

export type CreateCustomListDialogImplProps = Omit<
  CreateCustomListDialogProps,
  'open' | 'onOpenChange' | 'loading' | 'onLoadingChange'
>;

function CreateCustomListDialogImpl(props: CreateCustomListDialogImplProps) {
  const descriptionId = React.useId();

  const {
    open,
    onOpenChange,
    formRef,
    inputRef,
    loading,
    form: {
      error,
      customListTextField: { value, invalid, dirty, invalidReason, reset },
    },
  } = useCreateCustomListDialogContext();

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

  const textFieldInvalid = dirty && (error || invalid);
  const saveButtonDisabled = textFieldInvalid || loading || !open || !dirty;

  return (
    <Dialog open={open} onOpenChange={handleOnOpenChange} {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <form ref={formRef} onSubmit={handleSubmit}>
              <TextField
                value={value}
                onValueChange={handleOnValueChange}
                invalid={textFieldInvalid}>
                <Dialog.TextGroup gap="small" flexGrow="1">
                  <TextField.Label color="whiteAlpha60">
                    {
                      // TRANSLATORS: Label for the input where the user can enter the name of a new custom list.
                      messages.pgettext('custom-list-feature', 'Create custom list')
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
                            'custom-list-feature',
                            'Enter a name for the custom list',
                          )}
                    </Dialog.Text>
                  </FlexColumn>
                </Dialog.TextGroup>
              </TextField>
            </form>
            <Dialog.ButtonGroup>
              <Dialog.Button key="save" disabled={saveButtonDisabled} onClick={handleClick}>
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

export function CreateCustomListDialog({
  open,
  onOpenChange,
  loading,
  onLoadingChange,
  location,
  ...props
}: CreateCustomListDialogProps) {
  return (
    <CreateCustomListDialogProvider
      open={open}
      onOpenChange={onOpenChange}
      loading={loading}
      onLoadingChange={onLoadingChange}
      location={location}>
      <CreateCustomListDialogImpl {...props} />
    </CreateCustomListDialogProvider>
  );
}
