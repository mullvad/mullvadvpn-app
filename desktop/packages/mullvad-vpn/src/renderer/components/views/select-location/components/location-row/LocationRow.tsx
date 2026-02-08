import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { compareRelayLocation, RelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { LocationListItem } from '../../../../location-list-item';
import { type AnyLocation, getLocationChildrenByType } from '../../select-location-types';
import {
  AddToCustomListButton,
  DeleteCustomListButton,
  EditCustomListButton,
  RemoveFromCustomListButton,
} from './components';
import { LocationRowProvider, useLocationRowContext } from './LocationRowContext';

export type LocationRowProps = React.PropsWithChildren<{
  location: AnyLocation;
  level: ListItemProps['level'];
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: RelayLocation) => void;
  allowAddToCustomList: boolean;
}>;

function LocationRowImpl({
  allowAddToCustomList,
  level,
  onSelect,
  selectedElementRef,
  children,
}: Omit<LocationRowProps, 'location'>) {
  const { location } = useLocationRowContext();

  const childLocations = getLocationChildrenByType(location);
  const hasChildren = childLocations.some((child) => child.visible);

  const [expanded, setExpanded] = React.useState(location.expanded ?? false);

  const handleClick = useCallback(() => {
    if (!location.selected) {
      onSelect(location.details);
    }
  }, [onSelect, location]);

  if (!location.visible) {
    return null;
  }

  // The selectedRef should only be used if the element is selected
  const selectedRef = location.selected ? selectedElementRef : undefined;
  return (
    <LocationListItem selected={location.selected}>
      <LocationListItem.Accordion
        expanded={expanded}
        onExpandedChange={setExpanded}
        disabled={location.disabled}>
        <LocationListItem.Header ref={selectedRef} level={level}>
          <LocationListItem.HeaderTrigger
            onClick={handleClick}
            disabled={location.disabled}
            aria-label={sprintf(messages.pgettext('accessibility', 'Connect to %(location)s'), {
              location: location.label,
            })}>
            <LocationListItem.HeaderItem>
              <LocationListItem.Title>{location.label}</LocationListItem.Title>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>

          <LocationListItem.HeaderTrailingActions>
            {allowAddToCustomList && location.type !== 'customList' ? (
              <AddToCustomListButton location={location} />
            ) : null}

            {/* Show remove from custom list button if location is top level item in a custom list. */}
            {location.type !== 'customList' &&
            location.details.customList !== undefined &&
            level === 1 ? (
              <RemoveFromCustomListButton location={location} />
            ) : null}
            {/* Show buttons for editing and removing a custom list */}
            {location.type === 'customList' ? (
              <>
                <EditCustomListButton customList={location} />
                <DeleteCustomListButton customList={location} />
              </>
            ) : null}
            {hasChildren || location.type === 'customList' ? (
              <LocationListItem.AccordionTrigger
                aria-label={sprintf(
                  location.expanded === true
                    ? messages.pgettext('accessibility', 'Collapse %(location)s')
                    : messages.pgettext('accessibility', 'Expand %(location)s'),
                  { location: location.label },
                )}>
                <LocationListItem.HeaderTrailingAction>
                  <LocationListItem.Icon />
                </LocationListItem.HeaderTrailingAction>
              </LocationListItem.AccordionTrigger>
            ) : null}
          </LocationListItem.HeaderTrailingActions>
        </LocationListItem.Header>

        {hasChildren && (
          <LocationListItem.AccordionContent>{children}</LocationListItem.AccordionContent>
        )}
      </LocationListItem.Accordion>
    </LocationListItem>
  );
}

function LocationRow({ location, ...props }: LocationRowProps) {
  return (
    <LocationRowProvider location={location}>
      <LocationRowImpl {...props} />
    </LocationRowProvider>
  );
}

// This is to avoid unnecessary rerenders since most of the subtree is hidden and would result in
// a lot more work than necessary
export default React.memo(LocationRow, compareProps);

function compareProps(oldProps: LocationRowProps, nextProps: LocationRowProps): boolean {
  return (
    oldProps.onSelect === nextProps.onSelect &&
    oldProps.allowAddToCustomList === nextProps.allowAddToCustomList &&
    compareLocation(oldProps.location, nextProps.location)
  );
}

function compareLocation(oldLocation: AnyLocation, nextLocation: AnyLocation): boolean {
  return (
    oldLocation.visible === nextLocation.visible &&
    oldLocation.label === nextLocation.label &&
    oldLocation.disabled === nextLocation.disabled &&
    oldLocation.selected === nextLocation.selected &&
    compareRelayLocation(oldLocation.details, nextLocation.details) &&
    compareExpanded(oldLocation, nextLocation) &&
    compareChildren(oldLocation, nextLocation)
  );
}

function compareChildren(oldLocation: AnyLocation, nextLocation: AnyLocation): boolean {
  const oldVisibleChildren = getLocationChildrenByType(oldLocation).filter(
    (child) => child.visible,
  );
  const nextVisibleChildren = getLocationChildrenByType(nextLocation).filter(
    (child) => child.visible,
  );

  // Children shouldn't be checked if the row is collapsed
  const nextExpanded = 'expanded' in nextLocation && nextLocation.expanded;

  return (
    (!nextExpanded && oldVisibleChildren.length > 0 && nextVisibleChildren.length > 0) ||
    (oldVisibleChildren.length === nextVisibleChildren.length &&
      oldVisibleChildren.every((oldChild, i) => compareLocation(oldChild, nextVisibleChildren[i])))
  );
}

function compareExpanded(oldLocation: AnyLocation, nextLocation: AnyLocation): boolean {
  const oldExpanded = 'expanded' in oldLocation && oldLocation.expanded;
  const nextExpanded = 'expanded' in nextLocation && nextLocation.expanded;
  return oldExpanded === nextExpanded;
}
