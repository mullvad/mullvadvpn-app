import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { type CustomListLocation } from '../../../../../../../features/location/types';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { LocationListItem } from '../../../../../../location-list-item';
import { DeleteCustomListButton, EditCustomListButton } from '..';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function CustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  const { expanded } = useAccordionContext();

  return (
    <LocationListItem.HeaderTrailingActions>
      <EditCustomListButton customList={customList} />
      <DeleteCustomListButton customList={customList} />
      <LocationListItem.AccordionTrigger
        aria-label={sprintf(
          expanded === true
            ? messages.pgettext('accessibility', 'Collapse %(location)s')
            : messages.pgettext('accessibility', 'Expand %(location)s'),
          { location: customList.label },
        )}>
        <LocationListItem.HeaderTrailingAction>
          <LocationListItem.HeaderChevron />
        </LocationListItem.HeaderTrailingAction>
      </LocationListItem.AccordionTrigger>
    </LocationListItem.HeaderTrailingActions>
  );
}
