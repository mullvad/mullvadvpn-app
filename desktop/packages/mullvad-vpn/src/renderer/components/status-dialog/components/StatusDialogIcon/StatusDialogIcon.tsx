import { Dialog } from '../../../../lib/components/dialog';
import { useStatusDialogContext } from '../../StatusDialogContext';

export function StatusDialogIcon() {
  const { variant } = useStatusDialogContext();

  switch (variant) {
    case 'caution':
      return <Dialog.Icon icon="alert-circle" />;
    case 'failure':
      return <Dialog.IconBadge state="negative" />;
    case 'info':
      return <Dialog.Icon icon="info-circle" />;
    case 'success':
      return <Dialog.IconBadge state="positive" />;
    case 'warning':
      return <Dialog.Icon icon="alert-circle" color="red" />;
    default:
      return variant satisfies never;
  }
}
