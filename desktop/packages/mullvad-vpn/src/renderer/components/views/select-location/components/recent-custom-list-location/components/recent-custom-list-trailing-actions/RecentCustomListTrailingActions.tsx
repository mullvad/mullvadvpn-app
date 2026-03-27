import React from 'react';

import {
  CustomListMenu,
  CustomListMenuButton,
} from '../../../../../../../features/custom-lists/components';
import type { CustomListLocation } from '../../../../../../../features/locations/types';
import { Location } from '../../../location-list-item';
import { useRecentCustomListLocationContext } from '../../RecentCustomListLocationContext';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function RecentCustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  const { loading, setLoading } = useRecentCustomListLocationContext();

  const customListMenuButtonRef = React.useRef<HTMLButtonElement>(null);
  const [customListMenuOpen, setCustomMenuOpen] = React.useState(false);
  const toggleCustomListMenu = React.useCallback(() => {
    setCustomMenuOpen((prev) => !prev);
  }, []);

  return (
    <Location.Accordion.Header.TrailingActions>
      <Location.Accordion.Header.TrailingActions.Action>
        <CustomListMenuButton
          ref={customListMenuButtonRef}
          customList={customList}
          onClick={toggleCustomListMenu}
        />
        <CustomListMenu
          triggerRef={customListMenuButtonRef}
          open={customListMenuOpen}
          onOpenChange={setCustomMenuOpen}
          customList={customList}
          loading={loading}
          setLoading={setLoading}
        />
      </Location.Accordion.Header.TrailingActions.Action>
    </Location.Accordion.Header.TrailingActions>
  );
}
