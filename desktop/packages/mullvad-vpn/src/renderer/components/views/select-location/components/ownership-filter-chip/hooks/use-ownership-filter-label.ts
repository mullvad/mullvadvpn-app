import { Ownership } from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';
import { useActiveOwnership } from '../../../hooks';

export const useOwnershipFilterLabel = () => {
  const { activeOwnership } = useActiveOwnership();

  if (activeOwnership === Ownership.mullvadOwned) {
    return messages.pgettext('filter-view', 'Owned');
  } else if (activeOwnership === Ownership.rented) {
    return messages.pgettext('filter-view', 'Rented');
  } else {
    return '';
  }
};
