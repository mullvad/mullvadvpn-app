import { useBoolean } from '../../../../../../../lib/utility-hooks';
import { LocationListItem } from '../../../../../../location-list-item';
import type { LocationListItemIconButtonProps } from '../../../../../../location-list-item/components';
import { EditListDialog } from '../../../edit-list-dialog';
import { useLocationRowContext } from '../../LocationRowContext';

export type EditCustomListButtonProps = LocationListItemIconButtonProps;

export function EditCustomListButton(props: EditCustomListButtonProps) {
  const [editDialogVisible, showEditDialog, hideEditDialog] = useBoolean();
  const { source } = useLocationRowContext();

  return (
    <>
      <LocationListItem.IconButton onClick={showEditDialog} variant="secondary" {...props}>
        <LocationListItem.IconButton.Icon icon="edit-circle" />
      </LocationListItem.IconButton>

      {'list' in source && (
        <EditListDialog list={source.list} isOpen={editDialogVisible} hide={hideEditDialog} />
      )}
    </>
  );
}
