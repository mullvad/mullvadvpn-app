import { Dialog } from '../../../../lib/components/dialog';
import type { ApiAccessMethodTestingState } from '../types';

export function getTestingDialogStatusIcon(state: ApiAccessMethodTestingState) {
  switch (state) {
    case 'testing':
      return <Dialog.Spinner />;
    case 'success':
      return <Dialog.IconBadge state="positive" />;
    case 'failure':
      return <Dialog.IconBadge state="negative" />;
  }
}
