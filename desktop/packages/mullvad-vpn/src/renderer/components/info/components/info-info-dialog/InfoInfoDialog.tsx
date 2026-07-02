import { messages } from '../../../../../shared/gettext';
import { InfoDialog, type InfoDialogProps } from '../../../info-dialog';
import { useInfoContext } from '../../InfoContext';

export type InfoInfoDialogProps = InfoDialogProps;

function InfoInfoDialog({ children, ...props }: InfoInfoDialogProps) {
  const { open, onOpenChange } = useInfoContext();

  return (
    <InfoDialog open={open} onOpenChange={onOpenChange} {...props}>
      {children}
      <InfoDialog.CloseButton>
        <InfoDialog.CloseButton.Text>{messages.gettext('Got it!')}</InfoDialog.CloseButton.Text>
      </InfoDialog.CloseButton>
    </InfoDialog>
  );
}

const InfoInfoDialogNamespace = Object.assign(InfoInfoDialog, {
  Text: InfoDialog.Text,
  Title: InfoDialog.Title,
});

export { InfoInfoDialogNamespace as InfoInfoDialog };
