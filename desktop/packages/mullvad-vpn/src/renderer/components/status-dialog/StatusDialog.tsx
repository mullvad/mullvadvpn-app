import { Dialog, type DialogProps } from '../../lib/components/dialog';
import { StatusDialogIcon } from './components';
import { StatusDialogProvider } from './StatusDialogContext';

export type StatusDialogProps = DialogProps & {
  variant: 'caution' | 'failure' | 'info' | 'success' | 'warning';
};

function StatusDialog({ children, variant, ...props }: StatusDialogProps) {
  return (
    <StatusDialogProvider variant={variant}>
      <Dialog {...props}>
        <Dialog.Portal>
          <Dialog.Popup>
            <Dialog.PopupContent>
              <StatusDialogIcon />
              {children}
            </Dialog.PopupContent>
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog>
    </StatusDialogProvider>
  );
}

const StatusDialogNamespace = Object.assign(StatusDialog, {
  ButtonGroup: Dialog.ButtonGroup,
  CloseButton: Dialog.CloseButton,
  TextGroup: Dialog.TextGroup,
  Text: Dialog.Text,
  Title: Dialog.Title,
  Subtitle: Dialog.Subtitle,
  Button: Dialog.Button,
  List: Dialog.List,
});

export { StatusDialogNamespace as StatusDialog };
