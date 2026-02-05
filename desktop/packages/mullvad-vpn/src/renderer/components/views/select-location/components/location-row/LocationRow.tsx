import React, { useCallback, useRef } from 'react';
import { sprintf } from 'sprintf-js';

import { compareRelayLocation, RelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { LocationListItem } from '../../../../location-list-item';
import {
  type CitySpecification,
  type CountrySpecification,
  getLocationChildren,
  type LocationSpecification,
  type RelaySpecification,
} from '../../select-location-types';
import {
  AddToCustomListButton,
  DeleteCustomListButton,
  EditCustomListButton,
  RemoveFromCustomListButton,
} from './components';
import { LocationRowProvider, useLocationRowContext } from './LocationRowContext';

interface IProps<C extends LocationSpecification> {
  source: C;
  level: ListItemProps['level'];
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: RelayLocation) => void;
  onExpand: (location: RelayLocation) => void;
  onCollapse: (location: RelayLocation) => void;
  allowAddToCustomList: boolean;
  onWillExpand: (
    locationRect: DOMRect,
    expandedContentHeight: number,
    invokedByUser: boolean,
  ) => void;
  onTransitionEnd: () => void;
  children?: C extends RelaySpecification
    ? never
    : React.ReactElement<
        IProps<C extends CountrySpecification ? CitySpecification : RelaySpecification>
      >[];
}

function LocationRowImpl(props: Omit<IProps<LocationSpecification>, 'source'>) {
  const { onSelect } = props;
  const { source } = useLocationRowContext();

  const children = getLocationChildren(source);
  const hasChildren = children.some((child) => child.visible);
  const userInvokedExpand = useRef(false);

  // Expand/collapse should only be available if the expanded property is provided in the location
  const expanded = 'expanded' in source ? source.expanded : undefined;
  const handleExpandedChange = React.useCallback(() => {
    if (expanded !== undefined && hasChildren) {
      userInvokedExpand.current = true;
      const callback = expanded ? props.onCollapse : props.onExpand;
      callback(source.location);
    }
  }, [expanded, hasChildren, props.onCollapse, props.onExpand, source.location]);

  const handleClick = useCallback(() => {
    if (!source.selected) {
      onSelect(source.location);
    }
  }, [onSelect, source]);

  if (!source.visible) {
    return null;
  }

  // The selectedRef should only be used if the element is selected
  const selectedRef = source.selected ? props.selectedElementRef : undefined;
  return (
    <LocationListItem selected={source.selected}>
      <LocationListItem.Accordion
        expanded={expanded}
        onExpandedChange={handleExpandedChange}
        disabled={source.disabled}>
        <LocationListItem.Header ref={selectedRef} level={props.level}>
          <LocationListItem.HeaderTrigger
            onClick={handleClick}
            disabled={source.disabled}
            aria-label={sprintf(messages.pgettext('accessibility', 'Connect to %(location)s'), {
              location: source.label,
            })}>
            <LocationListItem.HeaderItem>
              <LocationListItem.Title>{source.label}</LocationListItem.Title>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>

          <LocationListItem.HeaderTrailingActions>
            {props.allowAddToCustomList ? <AddToCustomListButton /> : null}

            {/* Show remove from custom list button if location is top level item in a custom list. */}
            {'customList' in source.location &&
            'country' in source.location &&
            props.level === 1 ? (
              <RemoveFromCustomListButton />
            ) : null}
            {/* Show buttons for editing and removing a custom list */}
            {'customList' in source.location && !('country' in source.location) ? (
              <>
                <EditCustomListButton />
                <DeleteCustomListButton />
              </>
            ) : null}
            {hasChildren || ('customList' in source.location && !('country' in source.location)) ? (
              <LocationListItem.AccordionTrigger
                aria-label={sprintf(
                  expanded === true
                    ? messages.pgettext('accessibility', 'Collapse %(location)s')
                    : messages.pgettext('accessibility', 'Expand %(location)s'),
                  { location: source.label },
                )}>
                <LocationListItem.HeaderTrailingAction>
                  <LocationListItem.Icon />
                </LocationListItem.HeaderTrailingAction>
              </LocationListItem.AccordionTrigger>
            ) : null}
          </LocationListItem.HeaderTrailingActions>
        </LocationListItem.Header>

        {hasChildren && (
          <LocationListItem.AccordionContent>{props.children}</LocationListItem.AccordionContent>
        )}
      </LocationListItem.Accordion>
    </LocationListItem>
  );
}

function LocationRow({ source, ...props }: IProps<LocationSpecification>) {
  return (
    <LocationRowProvider source={source}>
      <LocationRowImpl {...props} />
    </LocationRowProvider>
  );
}

// This is to avoid unnecessary rerenders since most of the subtree is hidden and would result in
// a lot more work than necessary
export default React.memo(LocationRow, compareProps);

function compareProps<C extends LocationSpecification>(
  oldProps: IProps<C>,
  nextProps: IProps<C>,
): boolean {
  return (
    oldProps.onSelect === nextProps.onSelect &&
    oldProps.onExpand === nextProps.onExpand &&
    oldProps.onCollapse === nextProps.onCollapse &&
    oldProps.onWillExpand === nextProps.onWillExpand &&
    oldProps.onTransitionEnd === nextProps.onTransitionEnd &&
    oldProps.allowAddToCustomList === nextProps.allowAddToCustomList &&
    compareLocation(oldProps.source, nextProps.source)
  );
}

function compareLocation(
  oldLocation: LocationSpecification,
  nextLocation: LocationSpecification,
): boolean {
  return (
    oldLocation.visible === nextLocation.visible &&
    oldLocation.label === nextLocation.label &&
    oldLocation.active === nextLocation.active &&
    oldLocation.disabled === nextLocation.disabled &&
    oldLocation.selected === nextLocation.selected &&
    compareRelayLocation(oldLocation.location, nextLocation.location) &&
    compareExpanded(oldLocation, nextLocation) &&
    compareChildren(oldLocation, nextLocation)
  );
}

function compareChildren(
  oldLocation: LocationSpecification,
  nextLocation: LocationSpecification,
): boolean {
  const oldVisibleChildren = getLocationChildren(oldLocation).filter((child) => child.visible);
  const nextVisibleChildren = getLocationChildren(nextLocation).filter((child) => child.visible);

  // Children shouldn't be checked if the row is collapsed
  const nextExpanded = 'expanded' in nextLocation && nextLocation.expanded;

  return (
    (!nextExpanded && oldVisibleChildren.length > 0 && nextVisibleChildren.length > 0) ||
    (oldVisibleChildren.length === nextVisibleChildren.length &&
      oldVisibleChildren.every((oldChild, i) => compareLocation(oldChild, nextVisibleChildren[i])))
  );
}

function compareExpanded(
  oldLocation: LocationSpecification,
  nextLocation: LocationSpecification,
): boolean {
  const oldExpanded = 'expanded' in oldLocation && oldLocation.expanded;
  const nextExpanded = 'expanded' in nextLocation && nextLocation.expanded;
  return oldExpanded === nextExpanded;
}
