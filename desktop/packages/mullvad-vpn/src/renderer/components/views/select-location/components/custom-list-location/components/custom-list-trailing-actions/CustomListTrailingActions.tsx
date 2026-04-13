import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import {
  CustomListMenu,
  CustomListMenuButton,
} from '../../../../../../../features/custom-lists/components';
import { type CustomListLocation } from '../../../../../../../features/locations/types';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { Location } from '../../../location-list-item';
import { useCustomListLocationContext } from '../../CustomListLocationContext';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function CustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  const { expanded } = useAccordionContext();
  const { loading, setLoading } = useCustomListLocationContext();

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
      <Location.Accordion.Header.AccordionTrigger
        aria-label={sprintf(
          expanded
            ? messages.pgettext('accessibility', 'Collapse %(location)s')
            : messages.pgettext('accessibility', 'Expand %(location)s'),
          { location: customList.label },
        )}>
        <Location.Accordion.Header.TrailingActions.Action>
          <Location.Accordion.Header.TrailingActions.Action.Chevron />
        </Location.Accordion.Header.TrailingActions.Action>
      </Location.Accordion.Header.AccordionTrigger>
    </Location.Accordion.Header.TrailingActions>
  );
}
