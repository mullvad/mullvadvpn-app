import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useActiveFilters } from '../../../../../features/locations/hooks';
import { LocationType } from '../../../../../features/locations/types';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { LocationSelector } from '../../../../../lib/components/location-selector';
import { useIsLocationSelectorExpanded, useIsLocationSelectorIsolated } from '../../hooks';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { FilterChips } from '../filter-chips';
import { SelectLocationSelectorEntryItem, SelectLocationSelectorExitItem } from './components';
import {
  useCollapsibleContentHeight,
  useHandleSelectedItemChange,
  useLocationSelectorVariant,
  useShowSelectLocationSelectorEntryItem,
  useShowSelectLocationSelectorExitItem,
} from './hooks';

export function SelectLocationSelector() {
  const { locationType } = useSelectLocationViewContext();
  const { isAnyFilterActive: showFilterChips } = useActiveFilters(locationType);

  const handleSelectedItemChange = useHandleSelectedItemChange();
  const showSelectLocationSelectorEntryItem = useShowSelectLocationSelectorEntryItem();
  const showSelectLocationSelectorExitItem = useShowSelectLocationSelectorExitItem();
  const variant = useLocationSelectorVariant();

  const topRowContentRef = React.useRef<HTMLDivElement>(null);
  const bottomRowContentRef = React.useRef<HTMLDivElement>(null);

  const collapsibleContentHeight = useCollapsibleContentHeight(
    topRowContentRef,
    bottomRowContentRef,
  );
  const isLocationSelectorExpanded = useIsLocationSelectorExpanded(collapsibleContentHeight);
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  const selectedItem = locationType === LocationType.entry ? 'entry' : 'exit';

  return (
    <LocationSelector
      selectedItem={selectedItem}
      onSelectedItemChange={handleSelectedItemChange}
      expanded={isLocationSelectorExpanded && !isLocationSelectorIsolated}
      variant={variant}>
      <LocationSelector.Row position="top">
        <FlexColumn ref={topRowContentRef}>
          <LocationSelector.Row.Content>
            <LocationSelector.Row.Icon icon="device" />
            <LocationSelector.Row.Label>
              {messages.gettext('Your device')}
            </LocationSelector.Row.Label>
          </LocationSelector.Row.Content>
        </FlexColumn>
      </LocationSelector.Row>
      <LocationSelector.Items>
        {/* NOTE: The components must have a `key` assigned as the `LocationSelector.Items`
         * component uses `motion` components under the hood, which requires all children
         * to use keys.
         */}
        {showSelectLocationSelectorEntryItem ? (
          <SelectLocationSelectorEntryItem key="entry" type="entry" />
        ) : null}
        {showSelectLocationSelectorExitItem ? (
          <SelectLocationSelectorExitItem key="exit" type="exit" />
        ) : null}
      </LocationSelector.Items>
      <LocationSelector.Row position="bottom">
        <FlexColumn gap="small" ref={bottomRowContentRef}>
          <LocationSelector.Row.Content>
            <LocationSelector.Row.Icon icon="internet" />
            <LocationSelector.Row.Label>{messages.gettext('Internet')}</LocationSelector.Row.Label>
          </LocationSelector.Row.Content>
          {showFilterChips && <FilterChips />}
        </FlexColumn>
      </LocationSelector.Row>
    </LocationSelector>
  );
}
