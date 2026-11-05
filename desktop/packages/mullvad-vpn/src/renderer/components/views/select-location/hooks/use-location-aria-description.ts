import { messages } from '../../../../../shared/gettext';
import type { AnyLocation } from '../../../../features/locations/types';

export function useLocationAriaDescription(location: AnyLocation) {
  if (location.selected) {
    return messages.gettext('Selected');
  } else return undefined;
}
