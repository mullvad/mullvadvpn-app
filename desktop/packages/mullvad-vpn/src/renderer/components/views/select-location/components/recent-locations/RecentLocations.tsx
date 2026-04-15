import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { RecentLocationList } from './components';

export function RecentLocations() {
  const titleId = React.useId();

  return (
    <FlexColumn as="section" gap="tiny" margin={{ bottom: 'large' }} aria-labelledby={titleId}>
      <SectionTitle>
        <SectionTitle.Title as="h3" id={titleId}>
          {
            // TRANSLATORS: Title for section showing recently used locations.
            messages.pgettext('select-location-view', 'Recents')
          }
        </SectionTitle.Title>
        <SectionTitle.Divider />
      </SectionTitle>
      <RecentLocationList />
    </FlexColumn>
  );
}
