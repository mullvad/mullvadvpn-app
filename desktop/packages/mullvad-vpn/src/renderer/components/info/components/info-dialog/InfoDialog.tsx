import { messages } from '../../../../../shared/gettext';
import { Dialog } from '../../../../lib/components/dialog';
import { useInfoContext } from '../../InfoContext';

export type InfoDialogProps = React.PropsWithChildren;

function InfoDialog({ children, ...props }: InfoDialogProps) {
  const { open, onOpenChange } = useInfoContext();

  return (
    <Dialog open={open} onOpenChange={onOpenChange} {...props}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Icon icon="info-circle" />
            {children}
            <Dialog.CloseButton>
              <Dialog.CloseButton.Text>{messages.gettext('Got it!')}</Dialog.CloseButton.Text>
            </Dialog.CloseButton>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}

const InfoDialogNamespace = Object.assign(InfoDialog, {
  Text: Dialog.Text,
  Title: Dialog.Title,
});

export { InfoDialogNamespace as InfoDialog };
