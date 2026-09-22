import { Ownership } from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';
import { useOwnership } from '../../../../../../features/locations/hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export const useOwnershipFilterLabel = () => {
  const { locationType } = useSelectLocationViewContext();
  const { ownership } = useOwnership(locationType);

  console.log('ownership', ownership);
  if (ownership === Ownership.mullvadOwned) {
    return messages.pgettext('filter-view', 'Owned');
  } else if (ownership === Ownership.rented) {
    return messages.pgettext('filter-view', 'Rented');
  } else {
    return '';
  }
};
