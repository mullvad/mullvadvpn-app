import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { RemoveLocationFromCustomListButton } from '../../../../../features/custom-lists/components';
import { type GeographicalLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { getLocationListItemMapProps } from '../../utils';
import { Location } from '../location-list-item';
import {
  CustomListGeographicalLocationProvider,
  useCustomListGeographicalLocationContext,
} from './CustomListGeographicalLocationContext';
import { useHandleSelectLocationInCustomList } from './hooks';
export type CustomListGeographicalLocationProps = Pick<ListItemProps, 'level' | 'position'> & {
  disabled?: boolean;
  location: GeographicalLocation;
};

function CustomListGeographicalLocationImpl({
  location,
  level,
  disabled,
  position,
}: CustomListGeographicalLocationProps) {
  const { loading, setLoading } = useCustomListGeographicalLocationContext();
  const [expanded, setExpanded] = useState(location.expanded);

  const locationChildren = getLocationChildren(location);
  const showChildren = locationChildren.length > 0 && expanded;
  // Show remove from custom list button if location is top level item in a custom list.
  const showRemoveFromCustomListButton = level === 1;
  const showAccordionTrigger = locationChildren.length > 0;

  useEffect(() => {
    setExpanded(location.expanded);
  }, [location.expanded]);

  const handleSelectLocationInCustomList = useHandleSelectLocationInCustomList();

  const handleClick = useCallback(() => {
    void handleSelectLocationInCustomList(location);
  }, [location, handleSelectLocationInCustomList]);

  const children = locationChildren.map((locationChild) => {
    const { key, nextLevel } = getLocationListItemMapProps(locationChild, level);
    return (
      <CustomListGeographicalLocation
        key={key}
        location={locationChild}
        level={nextLevel}
        disabled={disabled}
        position={position}
      />
    );
  });

  return (
    <Location selected={location.selected}>
      <Location.Accordion
        expanded={expanded}
        onExpandedChange={setExpanded}
        disabled={location.disabled || disabled}>
        <Location.Accordion.Header level={level} position={position}>
          <Location.Accordion.Header.Trigger
            onClick={handleClick}
            aria-label={sprintf(
              // TRANSLATORS: Accessibility label for a button that connects to a location.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
              messages.pgettext('accessibility', 'Connect to %(location)s'),
              {
                location: location.label,
              },
            )}>
            <Location.Accordion.Header.Item>
              <Location.Accordion.Header.Item.Title>
                {location.label}
              </Location.Accordion.Header.Item.Title>
            </Location.Accordion.Header.Item>
          </Location.Accordion.Header.Trigger>
          <Location.Accordion.Header.TrailingActions>
            {showRemoveFromCustomListButton && (
              <Location.Accordion.Header.TrailingActions.Action>
                <RemoveLocationFromCustomListButton
                  location={location}
                  loading={loading}
                  onLoadingChange={setLoading}
                />
              </Location.Accordion.Header.TrailingActions.Action>
            )}
            {showAccordionTrigger && (
              <Location.Accordion.Trigger
                aria-label={sprintf(
                  expanded
                    ? messages.pgettext('accessibility', 'Collapse %(location)s')
                    : messages.pgettext('accessibility', 'Expand %(location)s'),
                  { location: location.label },
                )}>
                <Location.Accordion.Header.TrailingActions.Action>
                  <Location.Accordion.Header.TrailingActions.Action.Icon
                    icon={expanded ? 'chevron-up' : 'chevron-down'}
                  />
                </Location.Accordion.Header.TrailingActions.Action>
              </Location.Accordion.Trigger>
            )}
          </Location.Accordion.Header.TrailingActions>
        </Location.Accordion.Header>
        <Location.Accordion.Content>
          <AnimatedList>{showChildren ? children : null}</AnimatedList>
        </Location.Accordion.Content>
      </Location.Accordion>
    </Location>
  );
}

export function CustomListGeographicalLocation({ ...props }: CustomListGeographicalLocationProps) {
  return (
    <CustomListGeographicalLocationProvider>
      <CustomListGeographicalLocationImpl {...props} />
    </CustomListGeographicalLocationProvider>
  );
}
