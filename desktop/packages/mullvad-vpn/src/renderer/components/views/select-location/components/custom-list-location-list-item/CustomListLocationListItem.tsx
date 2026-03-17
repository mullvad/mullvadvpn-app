import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { RemoveLocationFromCustomListButton } from '../../../../../features/custom-lists/components';
import { type GeographicalLocation } from '../../../../../features/locations/types';
import { getLocationChildren } from '../../../../../features/locations/utils';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { getLocationListItemMapProps } from '../../utils';
import { LocationListItem } from '../location-list-item';
import {
  CustomListLocationListItemProvider,
  useCustomListLocationListItemContext,
} from './CustomListLocationListItemContext';
import { useHandleSelectCustomList } from './hooks';
export type CustomListLocationListItemProps = Pick<ListItemProps, 'level' | 'position'> & {
  disabled?: boolean;
  location: GeographicalLocation;
};

function CustomListLocationListItemImpl({
  location,
  level,
  disabled,
  position,
}: CustomListLocationListItemProps) {
  const { loading, setLoading } = useCustomListLocationListItemContext();
  const [expanded, setExpanded] = useState(location.expanded);

  const locationChildren = getLocationChildren(location);
  const showChildren = locationChildren.length > 0 && expanded;
  // Show remove from custom list button if location is top level item in a custom list.
  const showRemoveFromCustomListButton = level === 1;
  const showAccordionTrigger = locationChildren.length > 0;

  useEffect(() => {
    setExpanded(location.expanded);
  }, [location.expanded]);

  const handleSelectCustomList = useHandleSelectCustomList();

  const handleClick = useCallback(() => {
    void handleSelectCustomList(location);
  }, [location, handleSelectCustomList]);

  const children = locationChildren.map((locationChild) => {
    const { key, nextLevel } = getLocationListItemMapProps(locationChild, level);
    return (
      <CustomListLocationListItem
        key={key}
        location={locationChild}
        level={nextLevel}
        disabled={disabled}
        position={position}
      />
    );
  });

  return (
    <LocationListItem selected={location.selected}>
      <LocationListItem.Accordion
        expanded={expanded}
        onExpandedChange={setExpanded}
        disabled={location.disabled || disabled}>
        <LocationListItem.Header level={level} position={position}>
          <LocationListItem.HeaderTrigger
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
            <LocationListItem.HeaderItem>
              <LocationListItem.HeaderTitle>{location.label}</LocationListItem.HeaderTitle>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>
          <LocationListItem.HeaderTrailingActions>
            {showRemoveFromCustomListButton && (
              <LocationListItem.HeaderTrailingActions.Action>
                <RemoveLocationFromCustomListButton
                  location={location}
                  loading={loading}
                  onLoadingChange={setLoading}
                />
              </LocationListItem.HeaderTrailingActions.Action>
            )}
            {showAccordionTrigger && (
              <LocationListItem.AccordionTrigger
                aria-label={sprintf(
                  expanded === true
                    ? messages.pgettext('accessibility', 'Collapse %(location)s')
                    : messages.pgettext('accessibility', 'Expand %(location)s'),
                  { location: location.label },
                )}>
                <LocationListItem.HeaderTrailingActions.Action>
                  <LocationListItem.HeaderChevron />
                </LocationListItem.HeaderTrailingActions.Action>
              </LocationListItem.AccordionTrigger>
            )}
          </LocationListItem.HeaderTrailingActions>
        </LocationListItem.Header>
        <LocationListItem.AccordionContent>
          <AnimatedList>{showChildren ? children : null}</AnimatedList>
        </LocationListItem.AccordionContent>
      </LocationListItem.Accordion>
    </LocationListItem>
  );
}

export function CustomListLocationListItem({ ...props }: CustomListLocationListItemProps) {
  return (
    <CustomListLocationListItemProvider>
      <CustomListLocationListItemImpl {...props} />
    </CustomListLocationListItemProvider>
  );
}
