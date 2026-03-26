import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { Menu, type MenuProps } from '../../../../lib/components/menu';
import type { CustomListLocation } from '../../../locations/types';
import { DeleteCustomListDialog } from '../delete-custom-list-dialog';
import { EditCustomListDialog } from '../edit-custom-list-dialog';

export type CustomListMenuProps = MenuProps & {
  customList: CustomListLocation;
  loading?: boolean;
  setLoading?: (loading: boolean) => void;
};

export function CustomListMenu({
  onOpenChange,
  customList,
  loading,
  setLoading,
  ...props
}: CustomListMenuProps) {
  const [editCustomListDialogOpen, setEditCustomListDialogOpen] = React.useState(false);

  const showEditCustomListDialog = React.useCallback(() => {
    setEditCustomListDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange]);

  const [deleteCustomListDialogOpen, setDeleteCustomListDialogOpen] = React.useState(false);
  const showDeleteCustomListDialog = React.useCallback(() => {
    setDeleteCustomListDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange]);

  return (
    <>
      <Menu onOpenChange={onOpenChange} {...props}>
        <Menu.Popup>
          <Menu.Option>
            <Menu.Option.Trigger onClick={showEditCustomListDialog}>
              <Menu.Option.Item>
                <Menu.Option.Item.Icon icon="edit" />
                <Menu.Option.Item.Label>
                  {
                    // TRANSLATORS: Label for the option to edit a custom list in a menu.
                    messages.pgettext('custom-list-feature', 'Edit name')
                  }
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
          <Menu.Option>
            <Menu.Option.Trigger onClick={showDeleteCustomListDialog}>
              <Menu.Option.Item>
                <Menu.Option.Item.Icon icon="trash" />
                <Menu.Option.Item.Label>
                  {
                    // TRANSLATORS: Label for the option to delete a custom list in a menu.
                    messages.pgettext('custom-list-feature', 'Delete')
                  }
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
        </Menu.Popup>
      </Menu>
      <EditCustomListDialog
        customList={customList}
        open={editCustomListDialogOpen}
        onOpenChange={setEditCustomListDialogOpen}
        loading={loading}
        onLoadingChange={setLoading}
      />
      <DeleteCustomListDialog
        customList={customList}
        open={deleteCustomListDialogOpen}
        onOpenChange={setDeleteCustomListDialogOpen}
        loading={loading}
        onLoadingChange={setLoading}
      />
    </>
  );
}
