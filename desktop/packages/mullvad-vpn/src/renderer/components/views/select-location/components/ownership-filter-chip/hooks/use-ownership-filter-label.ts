import { Ownership } from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';

export const useOwnershipFilterLabel = (ownership: Ownership) => {
  if (ownership === Ownership.mullvadOwned) {
    return messages.pgettext('filter-view', 'Owned');
  } else if (ownership === Ownership.rented) {
    return messages.pgettext('filter-view', 'Rented');
  } else {
    return '';
  }
};
