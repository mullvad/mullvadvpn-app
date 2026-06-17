import { Dialog, type DialogProps } from '../../lib/components/dialog';

export type WarningDialogProps = DialogProps;

function WarningDialog({ children, ...props }: WarningDialogProps) {
  return (
    <Dialog {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="alert-circle" color="red" />
            {children}
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}

const WarningDialogNamespace = Object.assign(WarningDialog, {
  ButtonGroup: Dialog.ButtonGroup,
  CloseButton: Dialog.CloseButton,
  TextGroup: Dialog.TextGroup,
  Text: Dialog.Text,
  Title: Dialog.Title,
  Subtitle: Dialog.Subtitle,
  Button: Dialog.Button,
  List: Dialog.List,
});

export { WarningDialogNamespace as WarningDialog };
