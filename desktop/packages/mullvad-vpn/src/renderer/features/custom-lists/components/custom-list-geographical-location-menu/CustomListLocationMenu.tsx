import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { Menu, type MenuProps } from '../../../../lib/components/menu';
import type { GeographicalLocation } from '../../../locations/types';
import { useGetCustomListById, useRemoveLocationFromCustomList } from '../../hooks';

export type CustomListGeographicalLocationMenuProps = MenuProps & {
  location: GeographicalLocation;
  loading?: boolean;
  setLoading: (loading: boolean) => void;
};

export function CustomListGeographicalLocationMenu({
  onOpenChange,
  location,
  loading,
  setLoading,
  ...props
}: CustomListGeographicalLocationMenuProps) {
  const removeLocationFromCustomList = useRemoveLocationFromCustomList();
  const getCustomListById = useGetCustomListById();

  const handleOnClick = React.useCallback(async () => {
    const customListId = location.details.customList;
    if (customListId !== undefined) {
      setLoading(true);
      onOpenChange?.(false);
      const success = await removeLocationFromCustomList(customListId, location.details);

      // Only set loading to false if failed to keep disabled state while animating out
      if (!success) {
        setLoading(false);
      }
    }
  }, [location.details, onOpenChange, removeLocationFromCustomList, setLoading]);

  const customListId = location.details.customList;
  const customList = getCustomListById(customListId ?? '');
  const disabled = loading || customList === undefined;

  return (
    <Menu onOpenChange={onOpenChange} {...props}>
      <Menu.Popup>
        <Menu.Option disabled={disabled}>
          <Menu.Option.Trigger onClick={handleOnClick}>
            <Menu.Option.Item>
              <Menu.Option.Item.Icon icon="trash" />
              <Menu.Option.Item.Label>
                {messages.gettext('Remove from list')}
              </Menu.Option.Item.Label>
            </Menu.Option.Item>
          </Menu.Option.Trigger>
        </Menu.Option>
      </Menu.Popup>
    </Menu>
  );
}
