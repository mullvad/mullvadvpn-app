import { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import type { CustomListLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { useLocationAriaLabel } from '../../hooks';
import { Location } from '../location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { RecentCustomListTrailingActions } from './components';
import { RecentCustomListProvider } from './RecentCustomListLocationContext';

export type RecentCustomListLocationProps = {
  customList: CustomListLocation;
  disabled?: boolean;
  position?: ListItemProps['position'];
};

function RecentCustomListLocationImpl({
  customList,
  disabled: disabledProp,
  position,
}: RecentCustomListLocationProps) {
  const { handleSelect } = useLocationListsContext();

  const ariaLabel = useLocationAriaLabel(customList.label);

  const showEmptySubtitle = customList.locations.length === 0;
  const disabled = customList.disabled || disabledProp;

  const handleClick = useCallback(() => {
    void handleSelect(customList);
  }, [customList, handleSelect]);

  return (
    <Location root selected={customList.selected}>
      <Location.ListItem disabled={disabled} level={0} position={position}>
        <Location.ListItem.Trigger onClick={handleClick} aria-label={ariaLabel}>
          <Location.ListItem.Item>
            <FlexColumn>
              <Location.ListItem.Item.Label>{customList.label}</Location.ListItem.Item.Label>
              {showEmptySubtitle && (
                <FootnoteMiniSemiBold color="whiteAlpha60">
                  {
                    // TRANSLATORS: Label for custom lists that don't have any locations added to them yet.
                    messages.pgettext('select-location-view', 'Empty')
                  }
                </FootnoteMiniSemiBold>
              )}
            </FlexColumn>
          </Location.ListItem.Item>
        </Location.ListItem.Trigger>
        <RecentCustomListTrailingActions customList={customList} />
      </Location.ListItem>
    </Location>
  );
}

export function RecentCustomListLocation({ ...props }: RecentCustomListLocationProps) {
  return (
    <RecentCustomListProvider>
      <RecentCustomListLocationImpl {...props} />
    </RecentCustomListProvider>
  );
}
