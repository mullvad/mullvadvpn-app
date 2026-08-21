import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { SectionTitle } from '../../../../../lib/components/section-title';
import { getLocationListItemMapProps } from '../../utils';
import { CountryLocation } from '../country-location';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { useRelayCount } from './hooks';

export function CountryLocations() {
  const { countryLocations } = useLocationListsContext();
  const { visibleRelays, totalRelays } = useRelayCount();

  const titleId = React.useId();
  const descriptionId = React.useId();

  const showFilterText = visibleRelays !== totalRelays;

  return (
    <FlexColumn
      as="section"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      gap="tiny"
      tabIndex={-1}
      data-focusable-heading>
      <SectionTitle>
        <SectionTitle.Title as="h3" id={titleId} tabIndex={-1}>
          {messages.pgettext('select-location-view', 'All locations')}
        </SectionTitle.Title>
        <SectionTitle.Divider />
        {showFilterText && (
          <SectionTitle.Text id={descriptionId}>
            {sprintf(
              // TRANSLATORS: Text showing how many locations are currently shown out of the total number of locations, e.g. "Showing 5 of 250"
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(visibleRelays)s: The number of relays currently shown
              // TRANSLATORS: %(totalRelays)s: The total number of relays
              messages.pgettext(
                'select-location-view',
                'Showing %(visibleRelays)s of %(totalRelays)s',
              ),
              {
                visibleRelays,
                totalRelays,
              },
            )}
          </SectionTitle.Text>
        )}
      </SectionTitle>
      <FlexColumn>
        {countryLocations.map((location) => {
          const { key } = getLocationListItemMapProps(location, undefined);
          return <CountryLocation key={key} location={location} />;
        })}
      </FlexColumn>
    </FlexColumn>
  );
}
