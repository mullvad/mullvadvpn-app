import { messages } from '../../../../../shared/gettext';
import { Dialog } from '../../../../lib/components/dialog';
import type { ApiAccessMethodTestingState } from '../types';

export function getTestingDialogTitle(type: ApiAccessMethodTestingState, newMethod: boolean) {
  let title = '';
  switch (type) {
    case 'success':
      title = newMethod
        ? messages.pgettext('api-access-methods-view', 'API reachable, adding method…')
        : messages.pgettext('api-access-methods-view', 'API reachable, saving method…');
      break;
    case 'failure':
      title = newMethod
        ? messages.pgettext('api-access-methods-view', 'API unreachable, add anyway?')
        : messages.pgettext('api-access-methods-view', 'API unreachable, save anyway?');
      break;
    default:
    case 'testing':
      title = messages.pgettext('api-access-methods-view', 'Testing method...');
  }
  return <Dialog.Subtitle>{title}</Dialog.Subtitle>;
}
