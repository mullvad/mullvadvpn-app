import React from 'react';

import { RoutePath } from '../../../../shared/routes';
import { useActiveFilters } from '../../../features/locations/hooks';
import { LocationType } from '../../../features/locations/types';
import { LocationSelector } from '../../../lib/components/location-selector';
import { useHistory } from '../../../lib/history';

export function LocationSelectorTest() {
  const [selectedItem, setSelectedItem] = React.useState<string | undefined>('0');
  const [isolatedItem, setIsolatedItem] = React.useState<string | undefined>(undefined);
  const [placeholderData, setPlaceholderData] = React.useState<Record<string, string | undefined>>({
    '0': undefined,
    '1': undefined,
  });

  const handleSelectedItemChange = React.useCallback((itemId: string) => {
    setSelectedItem(itemId);
  }, []);

  const handleOnValueChange = React.useCallback(
    (item: string, value: string) => {
      if (value && !isolatedItem) {
        setIsolatedItem(item);
      } else if (!value && isolatedItem === item) {
        setIsolatedItem(undefined);
      }
      setPlaceholderData((prev) => ({ ...prev, [item]: value }));
    },
    [isolatedItem],
  );

  const history = useHistory();
  const handleClickFilterEntry = React.useCallback(() => {
    // setSelectLocationView(LocationType.entry);
    history.push(RoutePath.filter, {
      options: [{ type: 'filter-view-options', locationType: LocationType.entry }],
    });
  }, [history]);

  const handleClickFilterExit = React.useCallback(() => {
    history.push(RoutePath.filter, {
      options: [{ type: 'filter-view-options', locationType: LocationType.exit }],
    });
  }, [history]);

  const entryFiltersActive = useActiveFilters(LocationType.entry);
  const showEntryFiltersActiveIcon =
    entryFiltersActive.isOwnershipFilterActive || entryFiltersActive.isProvidersFilterActive;

  const exitFiltersActive = useActiveFilters(LocationType.exit);
  const showExitFiltersActiveIcon =
    exitFiltersActive.isOwnershipFilterActive || exitFiltersActive.isProvidersFilterActive;

  console.log('exitFiltersActive', exitFiltersActive);

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={isolatedItem === undefined}
      variant={Object.keys(placeholderData).length > 1 ? 'secondary' : 'primary'}>
      <LocationSelector.Row position="top">
        <LocationSelector.Row.Icon icon="device" />
        <LocationSelector.Row.Label>Your device</LocationSelector.Row.Label>
      </LocationSelector.Row>
      <LocationSelector.Items>
        {(isolatedItem ? [isolatedItem] : Object.keys(placeholderData)).map((item, index) => {
          return (
            <LocationSelector.Items.Item key={item} id={item} type={index === 0 ? 'entry' : 'exit'}>
              <LocationSelector.Items.Item.TextField
                value={placeholderData[item]}
                onValueChange={handleOnValueChange}>
                <LocationSelector.Items.Item.TextField.Input placeholder={'Location ' + item} />
                <LocationSelector.Items.Item.TextField.ClearButton />
              </LocationSelector.Items.Item.TextField>
              {index === 0 ? (
                /* Entry */
                <LocationSelector.Items.Item.TrailingButton
                  onClick={handleClickFilterEntry}
                  visible={isolatedItem === undefined}>
                  <LocationSelector.Items.Item.TrailingButton.Icon
                    icon={showEntryFiltersActiveIcon ? 'filter-active' : 'filter'}
                  />
                </LocationSelector.Items.Item.TrailingButton>
              ) : (
                /* Exit */
                <LocationSelector.Items.Item.TrailingButton
                  onClick={handleClickFilterExit}
                  visible={isolatedItem === undefined}>
                  <LocationSelector.Items.Item.TrailingButton.Icon
                    icon={showExitFiltersActiveIcon ? 'filter-active' : 'filter'}
                  />
                </LocationSelector.Items.Item.TrailingButton>
              )}
            </LocationSelector.Items.Item>
          );
        })}
      </LocationSelector.Items>
      <LocationSelector.Row position="bottom">
        <LocationSelector.Row.Icon icon="device" />
        <LocationSelector.Row.Label>Internet</LocationSelector.Row.Label>
      </LocationSelector.Row>
    </LocationSelector>
  );
}
