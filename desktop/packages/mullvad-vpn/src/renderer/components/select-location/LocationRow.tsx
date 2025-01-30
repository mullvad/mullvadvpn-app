import React, { useCallback, useRef } from 'react';
import { sprintf } from 'sprintf-js';

import {
  compareRelayLocation,
  compareRelayLocationGeographical,
  RelayLocation,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { useBoolean, useStyledRef } from '../../lib/utility-hooks';
import { useSelector } from '../../redux/store';
import Accordion from '../Accordion';
import * as Cell from '../cell';
import ChevronButton from '../ChevronButton';
import RelayStatusIndicator from '../RelayStatusIndicator';
import { AddToListDialog, DeleteConfirmDialog, EditListDialog } from './CustomListDialogs';
import {
  getButtonColor,
  StyledHoverIcon,
  StyledHoverIconButton,
  StyledLocationRowButton,
  StyledLocationRowContainer,
  StyledLocationRowLabel,
} from './LocationRowStyles';
import {
  CitySpecification,
  CountrySpecification,
  getLocationChildren,
  LocationSpecification,
  RelaySpecification,
} from './select-location-types';

interface IProps<C extends LocationSpecification> {
  source: C;
  level: number;
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

// Renders the rows and its children for countries, cities and relays
function LocationRow<C extends LocationSpecification>(props: IProps<C>) {
  const { onSelect, onWillExpand: propsOnWillExpand } = props;

  const hasChildren = getLocationChildren(props.source).some((child) => child.visible);
  const buttonRef = useStyledRef<HTMLButtonElement>();
  const userInvokedExpand = useRef(false);

  const { updateCustomList, deleteCustomList } = useAppContext();
  const [addToListDialogVisible, showAddToListDialog, hideAddToListDialog] = useBoolean();
  const [editDialogVisible, showEditDialog, hideEditDialog] = useBoolean();
  const [deleteDialogVisible, showDeleteDialog, hideDeleteDialog] = useBoolean();
  const background = getButtonColor(props.source.selected, props.level, props.source.disabled);

  const customLists = useSelector((state) => state.settings.customLists);

  // Expand/collapse should only be available if the expanded property is provided in the source
  const expanded = 'expanded' in props.source ? props.source.expanded : undefined;
  const toggleCollapse = useCallback(() => {
    if (expanded !== undefined && hasChildren) {
      userInvokedExpand.current = true;
      const callback = expanded ? props.onCollapse : props.onExpand;
      callback(props.source.location);
    }
  }, [props.onExpand, props.onCollapse, props.source.location, expanded, hasChildren]);

  const handleClick = useCallback(() => {
    if (!props.source.selected) {
      onSelect(props.source.location);
    }
  }, [onSelect, props.source.location, props.source.selected]);

  const onWillExpand = useCallback(
    (nextHeight: number) => {
      const buttonRect = buttonRef.current?.getBoundingClientRect();
      if (expanded !== undefined && buttonRect) {
        propsOnWillExpand(buttonRect, nextHeight, userInvokedExpand.current);
        userInvokedExpand.current = false;
      }
    },
    [buttonRef, expanded, propsOnWillExpand],
  );

  const onRemoveFromList = useCallback(async () => {
    if (props.source.location.customList) {
      // Find the list and remove the location from it.
      const list = customLists.find((list) => list.id === props.source.location.customList);
      if (list !== undefined) {
        const updatedList = {
          ...list,
          locations: list.locations.filter((location) => {
            return !compareRelayLocationGeographical(location, props.source.location);
          }),
        };

        try {
          await updateCustomList(updatedList);
        } catch (e) {
          const error = e as Error;
          log.error(
            `Failed to edit custom list ${props.source.location.customList}: ${error.message}`,
          );
        }
      }
    }
  }, [customLists, props.source.location, updateCustomList]);

  // Remove an entire custom list.
  const confirmRemoveCustomList = useCallback(async () => {
    if (props.source.location.customList) {
      try {
        await deleteCustomList(props.source.location.customList);
      } catch (e) {
        const error = e as Error;
        log.error(
          `Failed to delete custom list ${props.source.location.customList}: ${error.message}`,
        );
      }
    }
  }, [deleteCustomList, props.source.location.customList]);

  if (!props.source.visible) {
    return null;
  }

  // The selectedRef should only be used if the element is selected
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  return (
    <>
      <StyledLocationRowContainer ref={selectedRef} disabled={props.source.disabled}>
        <StyledLocationRowButton
          as="button"
          ref={buttonRef}
          onClick={handleClick}
          $level={props.level}
          disabled={props.source.disabled}
          includeMarginBottomOnLast
          {...background}>
          <RelayStatusIndicator active={props.source.active} selected={props.source.selected} />
          <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
        </StyledLocationRowButton>

        {props.allowAddToCustomList ? (
          <StyledHoverIconButton onClick={showAddToListDialog} $isLast {...background}>
            <StyledHoverIcon icon="add-circle" />
          </StyledHoverIconButton>
        ) : null}

        {/* Show remove from custom list button if location is top level item in a custom list. */}
        {'customList' in props.source.location &&
        'country' in props.source.location &&
        props.level === 1 ? (
          <StyledHoverIconButton onClick={onRemoveFromList} $isLast {...background}>
            <StyledHoverIcon icon="remove-circle" />
          </StyledHoverIconButton>
        ) : null}

        {/* Show buttons for editing and removing a custom list */}
        {'customList' in props.source.location && !('country' in props.source.location) ? (
          <>
            <StyledHoverIconButton onClick={showEditDialog} {...background}>
              <StyledHoverIcon icon="edit-circle" />
            </StyledHoverIconButton>
            <StyledHoverIconButton onClick={showDeleteDialog} $isLast {...background}>
              <StyledHoverIcon icon="cross-circle" />
            </StyledHoverIconButton>
          </>
        ) : null}

        {hasChildren ||
        ('customList' in props.source.location && !('country' in props.source.location)) ? (
          <Cell.SideButton
            as={ChevronButton}
            onClick={toggleCollapse}
            disabled={!hasChildren}
            up={expanded ?? false}
            aria-label={sprintf(
              expanded === true
                ? messages.pgettext('accessibility', 'Collapse %(location)s')
                : messages.pgettext('accessibility', 'Expand %(location)s'),
              { location: props.source.label },
            )}
            {...background}
          />
        ) : null}
      </StyledLocationRowContainer>

      {hasChildren && (
        <Accordion
          expanded={expanded}
          onWillExpand={onWillExpand}
          onTransitionEnd={props.onTransitionEnd}
          animationDuration={150}>
          <Cell.Group $noMarginBottom>{props.children}</Cell.Group>
        </Accordion>
      )}

      {'country' in props.source.location && (
        <AddToListDialog
          isOpen={addToListDialogVisible}
          hide={hideAddToListDialog}
          location={props.source.location}
        />
      )}

      {'list' in props.source && (
        <EditListDialog list={props.source.list} isOpen={editDialogVisible} hide={hideEditDialog} />
      )}

      {'list' in props.source && (
        <DeleteConfirmDialog
          list={props.source.list}
          isOpen={deleteDialogVisible}
          hide={hideDeleteDialog}
          confirm={confirmRemoveCustomList}
        />
      )}
    </>
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
