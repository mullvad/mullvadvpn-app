import { Dialog, type DialogProps } from '../../lib/components/dialog';

export type FailureDialogProps = DialogProps;

function FailureDialog({ children, ...props }: FailureDialogProps) {
  return (
    <Dialog {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.IconBadge state="negative" />
            {children}
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}

const FailureDialogNamespace = Object.assign(FailureDialog, {
  ButtonGroup: Dialog.ButtonGroup,
  CloseButton: Dialog.CloseButton,
  TextGroup: Dialog.TextGroup,
  Text: Dialog.Text,
  Title: Dialog.Title,
  Subtitle: Dialog.Subtitle,
  Button: Dialog.Button,
  List: Dialog.List,
});

export { FailureDialogNamespace as FailureDialog };
