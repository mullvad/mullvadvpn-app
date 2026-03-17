import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import {
  DeleteCustomListDialog,
  EditCustomListDialog,
} from '../../../../../../../features/custom-lists/components';
import { type CustomListLocation } from '../../../../../../../features/locations/types';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { LocationListItem } from '../../../location-list-item';
import { useCustomListLocationListContext } from '../../CustomListLocationListContext';
import { DeleteCustomListButton, EditCustomListButton } from './components';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function CustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  const { expanded } = useAccordionContext();
  const { loading, setLoading } = useCustomListLocationListContext();

  const [editCustomListDialogOpen, setEditCustomListDialogOpen] = React.useState(false);
  const showEditCustomListDialog = React.useCallback(() => {
    setEditCustomListDialogOpen(true);
  }, []);

  const [deleteCustomListDialogOpen, setDeleteCustomListDialogOpen] = React.useState(false);
  const showDeleteCustomListDialog = React.useCallback(() => {
    setDeleteCustomListDialogOpen(true);
  }, []);

  return (
    <LocationListItem.HeaderTrailingActions>
      <LocationListItem.HeaderTrailingActions.Action>
        <EditCustomListButton customList={customList} onClick={showEditCustomListDialog} />
        <EditCustomListDialog
          customList={customList}
          open={editCustomListDialogOpen}
          onOpenChange={setEditCustomListDialogOpen}
          loading={loading}
          onLoadingChange={setLoading}
        />
      </LocationListItem.HeaderTrailingActions.Action>
      <LocationListItem.HeaderTrailingActions.Action>
        <DeleteCustomListButton customList={customList} onClick={showDeleteCustomListDialog} />
        <DeleteCustomListDialog
          customList={customList}
          open={deleteCustomListDialogOpen}
          onOpenChange={setDeleteCustomListDialogOpen}
          loading={loading}
          onLoadingChange={setLoading}
        />
      </LocationListItem.HeaderTrailingActions.Action>
      <LocationListItem.AccordionTrigger
        aria-label={sprintf(
          expanded === true
            ? messages.pgettext('accessibility', 'Collapse %(location)s')
            : messages.pgettext('accessibility', 'Expand %(location)s'),
          { location: customList.label },
        )}>
        <LocationListItem.HeaderTrailingActions.Action>
          <LocationListItem.HeaderChevron />
        </LocationListItem.HeaderTrailingActions.Action>
      </LocationListItem.AccordionTrigger>
    </LocationListItem.HeaderTrailingActions>
  );
}
