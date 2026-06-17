import { Dialog, type DialogProps } from '../../lib/components/dialog';

export type InfoDialogProps = DialogProps;

function InfoDialog({ children, ...props }: InfoDialogProps) {
  return (
    <Dialog {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="info-circle" />
            {children}
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}

const InfoDialogNamespace = Object.assign(InfoDialog, {
  ButtonGroup: Dialog.ButtonGroup,
  CloseButton: Dialog.CloseButton,
  TextGroup: Dialog.TextGroup,
  Text: Dialog.Text,
  Title: Dialog.Title,
  Subtitle: Dialog.Subtitle,
  Button: Dialog.Button,
  List: Dialog.List,
});

export { InfoDialogNamespace as InfoDialog };
