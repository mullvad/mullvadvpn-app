import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationListItem } from '../../../../../../location-list-item';
import { type CustomListLocation } from '../../../../select-location-types';
import { DeleteCustomListButton, EditCustomListButton } from '..';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function CustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  return (
    <LocationListItem.HeaderTrailingActions>
      <EditCustomListButton customList={customList} />
      <DeleteCustomListButton customList={customList} />
      <LocationListItem.AccordionTrigger
        aria-label={sprintf(
          customList.expanded === true
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
