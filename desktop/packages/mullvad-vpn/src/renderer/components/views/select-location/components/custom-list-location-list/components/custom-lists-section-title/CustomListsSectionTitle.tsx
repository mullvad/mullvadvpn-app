import React from 'react';

import { messages } from '../../../../../../../../shared/gettext';
import {
  SectionTitle,
  type SectionTitleProps,
} from '../../../../../../../lib/components/section-title';
import { useCustomListListContext } from '../../CustomListLocationListContext';

export type CustomListsSectionTitleProps = SectionTitleProps;

export function CustomListsSectionTitle({ ...props }: CustomListsSectionTitleProps) {
  const { setAddCustomListDialogOpen } = useCustomListListContext();

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
        aria-label={messages.pgettext('accessibility', 'Add a new custom list')}>
        <SectionTitle.IconButton.Icon icon="add" />
      </SectionTitle.IconButton>
    </SectionTitle>
  );
}
