import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import {
  DeleteCustomListButton,
  EditCustomListButton,
} from '../../../../../../../features/custom-lists/components';
import { type CustomListLocation } from '../../../../../../../features/locations/types';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { LocationListItem } from '../../../../../../location-list-item';
import { useCustomListLocationListItemContext } from '../../../custom-list-location-list-item/CustomListLocationListItemContext';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function CustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  const { expanded } = useAccordionContext();
  const { loading, setLoading } = useCustomListLocationListItemContext();

  return (
    <LocationListItem.HeaderTrailingActions>
      <LocationListItem.HeaderTrailingAction>
        <EditCustomListButton
          customList={customList}
          loading={loading}
          onLoadingChange={setLoading}
        />
      </LocationListItem.HeaderTrailingAction>
      <LocationListItem.HeaderTrailingAction>
        <DeleteCustomListButton
          customList={customList}
          loading={loading}
          onLoadingChange={setLoading}
        />
      </LocationListItem.HeaderTrailingAction>
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
