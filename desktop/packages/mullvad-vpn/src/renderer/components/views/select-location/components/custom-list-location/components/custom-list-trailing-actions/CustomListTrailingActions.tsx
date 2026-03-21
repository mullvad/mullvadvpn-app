import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../shared/gettext';
import {
  DeleteCustomListDialog,
  EditCustomListDialog,
} from '../../../../../../../features/custom-lists/components';
import { type CustomListLocation } from '../../../../../../../features/locations/types';
import { useAccordionContext } from '../../../../../../../lib/components/accordion/AccordionContext';
import { Location } from '../../../location-list-item';
import { useCustomListLocationContext } from '../../CustomListLocationContext';
import { DeleteCustomListButton, EditCustomListButton } from './components';

export type CustomListTrailingActionsProps = React.PropsWithChildren<{
  customList: CustomListLocation;
}>;

export function CustomListTrailingActions({ customList }: CustomListTrailingActionsProps) {
  const { expanded } = useAccordionContext();
  const { loading, setLoading } = useCustomListLocationContext();

  const [editCustomListDialogOpen, setEditCustomListDialogOpen] = React.useState(false);
  const showEditCustomListDialog = React.useCallback(() => {
    setEditCustomListDialogOpen(true);
  }, []);

  const [deleteCustomListDialogOpen, setDeleteCustomListDialogOpen] = React.useState(false);
  const showDeleteCustomListDialog = React.useCallback(() => {
    setDeleteCustomListDialogOpen(true);
  }, []);

  return (
    <Location.Accordion.Header.TrailingActions>
      <Location.Accordion.Header.TrailingActions.Action>
        <EditCustomListButton customList={customList} onClick={showEditCustomListDialog} />
        <EditCustomListDialog
          customList={customList}
          open={editCustomListDialogOpen}
          onOpenChange={setEditCustomListDialogOpen}
          loading={loading}
          onLoadingChange={setLoading}
        />
      </Location.Accordion.Header.TrailingActions.Action>
      <Location.Accordion.Header.TrailingActions.Action>
        <DeleteCustomListButton customList={customList} onClick={showDeleteCustomListDialog} />
        <DeleteCustomListDialog
          customList={customList}
          open={deleteCustomListDialogOpen}
          onOpenChange={setDeleteCustomListDialogOpen}
          loading={loading}
          onLoadingChange={setLoading}
        />
      </Location.Accordion.Header.TrailingActions.Action>
      <Location.Accordion.Trigger
        aria-label={sprintf(
          expanded
            ? messages.pgettext('accessibility', 'Collapse %(location)s')
            : messages.pgettext('accessibility', 'Expand %(location)s'),
          { location: customList.label },
        )}>
        <Location.Accordion.Header.TrailingActions.Action>
          <Location.Accordion.Header.TrailingActions.Action.Icon
            icon={expanded ? 'chevron-up' : 'chevron-down'}
          />
        </Location.Accordion.Header.TrailingActions.Action>
      </Location.Accordion.Trigger>
    </Location.Accordion.Header.TrailingActions>
  );
}
