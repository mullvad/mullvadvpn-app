import { Dialog, type DialogProps } from '../../lib/components/dialog';

export type CautionDialogProps = DialogProps;

function CautionDialog({ children, ...props }: CautionDialogProps) {
  return (
    <Dialog {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="alert-circle" />
            {children}
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}

const CautionDialogNamespace = Object.assign(CautionDialog, {
  ButtonGroup: Dialog.ButtonGroup,
  CloseButton: Dialog.CloseButton,
  TextGroup: Dialog.TextGroup,
  Text: Dialog.Text,
  Title: Dialog.Title,
  Subtitle: Dialog.Subtitle,
  Button: Dialog.Button,
  List: Dialog.List,
});

export { CautionDialogNamespace as CautionDialog };
