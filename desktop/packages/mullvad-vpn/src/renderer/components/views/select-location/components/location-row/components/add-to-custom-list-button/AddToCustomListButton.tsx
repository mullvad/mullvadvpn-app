import { useBoolean } from '../../../../../../../lib/utility-hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { AddToListDialog } from '../../../add-to-list-dialog';
import { useLocationRowContext } from '../../LocationRowContext';

export type AddToCustomListButtonProps = LocationListItemIconButtonProps;

export function AddToCustomListButton(props: AddToCustomListButtonProps) {
  const [addToListDialogVisible, showAddToListDialog, hideAddToListDialog] = useBoolean();
  const {
    source: { location },
  } = useLocationRowContext();

  return (
    <>
      <LocationListItem.IconButton onClick={showAddToListDialog} {...props}>
        <LocationListItem.IconButton.Icon icon="add-circle" />
      </LocationListItem.IconButton>

      {'country' in location && (
        <AddToListDialog
          isOpen={addToListDialogVisible}
          hide={hideAddToListDialog}
          location={location}
        />
      )}
    </>
  );
}
