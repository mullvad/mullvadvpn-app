import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import { CreateCustomListDialog } from '../../../../../../../features/custom-lists/components';
import {
  SectionTitle,
  type SectionTitleProps,
} from '../../../../../../../lib/components/section-title';
import { useCustomListLocationListsContext } from '../../CustomListLocationListsContext';

export type CustomListsSectionTitleProps = SectionTitleProps;

export function CustomListsSectionTitle({ ...props }: CustomListsSectionTitleProps) {
  const {
    addingCustomList,
    setAddingCustomList,
    addCustomListDialogOpen,
    setAddCustomListDialogOpen,
  } = useCustomListLocationListsContext();

  const handleOnClick = React.useCallback(() => {
    setAddCustomListDialogOpen(true);
  }, [setAddCustomListDialogOpen]);

  return (
    <SectionTitle {...props}>
      <SectionTitle.Title>
        {messages.pgettext('select-location-view', 'Custom lists')}
      </SectionTitle.Title>
      <SectionTitle.Divider />
      <SectionTitle.IconButton
        onClick={handleOnClick}
        aria-label={
          // TRANSLATORS: This is an accessibility label for a button that opens a dialog to create a new custom list.
          messages.pgettext('accessibility', 'Add a new custom list')
        }>
        <SectionTitle.IconButton.Icon icon="add" />
      </SectionTitle.IconButton>
      <CreateCustomListDialog
        open={addCustomListDialogOpen}
        onOpenChange={setAddCustomListDialogOpen}
        loading={addingCustomList}
        onLoadingChange={setAddingCustomList}
      />
    </SectionTitle>
  );
}
