import { messages } from '../../../../../../../shared/gettext';
import type { FromMultihopMode } from '../types';

export function getFromText(from: FromMultihopMode) {
  switch (from) {
    case 'on':
      return messages.gettext('On');
    case 'off':
      return messages.gettext('Off');
  }
}
