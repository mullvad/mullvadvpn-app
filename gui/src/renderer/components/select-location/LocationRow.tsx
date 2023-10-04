import React, { useCallback, useRef } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import {
  compareRelayLocation,
  compareRelayLocationGeographical,
  RelayLocation,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { useAppContext } from '../../context';
import { useBoolean } from '../../lib/utilityHooks';
import { useSelector } from '../../redux/store';
import Accordion from '../Accordion';
import * as Cell from '../cell';
import ChevronButton from '../ChevronButton';
import { measurements, normalText } from '../common-styles';
import RelayStatusIndicator from '../RelayStatusIndicator';
import { AddToListDialog, EditListDialog } from './CustomListDioalogs';
import {
  CitySpecification,
  CountrySpecification,
  getLocationChildren,
  LocationSpecification,
  RelaySpecification,
} from './select-location-types';

interface IButtonColorProps {
  backgroundColor: string;
  backgroundColorHover: string;
}

const buttonColor = (props: IButtonColorProps) => {
  return {
    backgroundColor: props.backgroundColor,
    ':not(:disabled):hover': {
      backgroundColor: props.backgroundColorHover,
    },
  };
};

export const StyledLocationRowContainer = styled(Cell.Container)({
  display: 'flex',
  padding: 0,
  background: 'none',
});

export const StyledLocationRowButton = styled(Cell.Row)(
  buttonColor,
  (props: IButtonColorProps & { level: number }) => {
    const paddingLeft = (props.level + 1) * 16 + 2;

    return {
      display: 'flex',
      flex: 1,
      overflow: 'hidden',
      border: 'none',
      padding: `0 10px 0 ${paddingLeft}px`,
      margin: 0,
    };
  },
);

export const StyledLocationRowIcon = styled.button(buttonColor, {
  position: 'relative',
  alignSelf: 'stretch',
  paddingLeft: measurements.viewMargin,
  paddingRight: measurements.viewMargin,

  '&::before': {
    content: '""',
    position: 'absolute',
    margin: 'auto',
    top: 0,
    left: 0,
    bottom: 0,
    height: '50%',
    width: '1px',
    backgroundColor: colors.darkBlue,
  },
});

export const StyledLocationRowLabel = styled(Cell.Label)(normalText, {
  flex: 1,
  minWidth: 0,
  fontWeight: 400,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

const StyledHoverIconButton = styled.button(
  buttonColor,
  (props: { isLast?: boolean } & IButtonColorProps) => ({
    flex: 0,
    display: 'none',
    padding: '0 10px',
    paddingRight: props.isLast ? '17px' : '10px',
    margin: 0,
    border: 0,
    height: measurements.rowMinHeight,
    appearance: 'none',

    ':not(:disabled):hover': {
      backgroundColor: props.backgroundColor,
    },
    [`${StyledLocationRowContainer}:hover &&`]: {
      display: 'block',
    },
    [`${StyledLocationRowButton}:hover ~ &&`]: {
      backgroundColor: props.backgroundColorHover,
    },
  }),
);

const StyledHoverIcon = styled(Cell.Icon).attrs({ width: 18, height: 18 })({
  [`${StyledHoverIconButton}:hover &&`]: {
    backgroundColor: colors.white,
  },
});

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
  const hasChildren = React.Children.count(props.children) > 0;
  const buttonRef = useRef<HTMLButtonElement>() as React.RefObject<HTMLButtonElement>;
  const userInvokedExpand = useRef(false);

  const { updateCustomList, deleteCustomList } = useAppContext();
  const [addToListDialogVisible, showAddToListDialog, hideAddToListDialog] = useBoolean();
  const [editDialogVisible, showEditDialog, hideEditDialog] = useBoolean();
  const background = getButtonColor(props.source.selected, props.level, props.source.disabled);

  const customLists = useSelector((state) => state.settings.customLists);

  // Expand/collapse should only be available if the expanded property is provided in the source
  const expanded = 'expanded' in props.source ? props.source.expanded : undefined;
  const toggleCollapse = useCallback(() => {
    if (expanded !== undefined) {
      userInvokedExpand.current = true;
      const callback = expanded ? props.onCollapse : props.onExpand;
      callback(props.source.location);
    }
  }, [props.onExpand, props.onCollapse, props.source.location, expanded]);

  const handleClick = useCallback(() => {
    if (!props.source.selected) {
      props.onSelect(props.source.location);
    }
  }, [props.onSelect, props.source.location, props.source.selected]);

  const onWillExpand = useCallback(
    (nextHeight: number) => {
      const buttonRect = buttonRef.current?.getBoundingClientRect();
      if (expanded !== undefined && buttonRect) {
        props.onWillExpand(buttonRect, nextHeight, userInvokedExpand.current);
        userInvokedExpand.current = false;
      }
    },
    [props.onWillExpand, expanded],
  );

  const onRemoveFromList = useCallback(() => {
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
        void updateCustomList(updatedList);
      }
    }
  }, [customLists, props.source.location]);

  // Remove an entire custom list.
  const onRemoveCustomList = useCallback(() => {
    if (props.source.location.customList) {
      void deleteCustomList(props.source.location.customList);
    }
  }, [props.source.location.customList]);

  // The selectedRef should only be used if the element is selected
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  return (
    <>
      <StyledLocationRowContainer ref={selectedRef} disabled={props.source.disabled}>
        <StyledLocationRowButton
          as="button"
          ref={buttonRef}
          onClick={handleClick}
          level={props.level}
          disabled={props.source.disabled}
          includeMarginBottomOnLast
          {...background}>
          <RelayStatusIndicator active={props.source.active} selected={props.source.selected} />
          <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
        </StyledLocationRowButton>

        {props.allowAddToCustomList ? (
          <StyledHoverIconButton onClick={showAddToListDialog} isLast {...background}>
            <StyledHoverIcon source="icon-add" />
          </StyledHoverIconButton>
        ) : null}

        {/* Show remove from custom list button if location is top level item in a custom list. */}
        {'customList' in props.source.location &&
        'country' in props.source.location &&
        props.level === 1 ? (
          <StyledHoverIconButton onClick={onRemoveFromList} isLast {...background}>
            <StyledHoverIcon source="icon-remove" />
          </StyledHoverIconButton>
        ) : null}

        {/* Show buttons for editing and removing a custom list */}
        {'customList' in props.source.location && !('country' in props.source.location) ? (
          <>
            <StyledHoverIconButton onClick={showEditDialog} {...background}>
              <StyledHoverIcon source="icon-edit" />
            </StyledHoverIconButton>
            <StyledHoverIconButton onClick={onRemoveCustomList} isLast {...background}>
              <StyledHoverIcon source="icon-close" />
            </StyledHoverIconButton>
          </>
        ) : null}

        {hasChildren ? (
          <StyledLocationRowIcon
            as={ChevronButton}
            onClick={toggleCollapse}
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
          <Cell.Group noMarginBottom>{props.children}</Cell.Group>
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
    </>
  );
}

// This is to avoid unnecessary rerenders since most of the subtree is hidden and would result in
// a lot more work than necessary
export default React.memo(LocationRow, compareProps);

export function getButtonColor(selected: boolean, level: number, disabled?: boolean) {
  let backgroundColor = colors.blue60;
  if (selected) {
    backgroundColor = colors.green;
  } else if (level === 1) {
    backgroundColor = colors.blue40;
  } else if (level === 2) {
    backgroundColor = colors.blue20;
  } else if (level === 3) {
    backgroundColor = colors.blue10;
  }

  return {
    backgroundColor,
    backgroundColorHover: selected || disabled ? backgroundColor : colors.blue80,
  };
}

function compareProps<C extends LocationSpecification>(
  oldProps: IProps<C>,
  nextProps: IProps<C>,
): boolean {
  return (
    oldProps.onSelect === nextProps.onSelect &&
    oldProps.onExpand === nextProps.onExpand &&
    oldProps.onWillExpand === nextProps.onWillExpand &&
    oldProps.onTransitionEnd === nextProps.onTransitionEnd &&
    compareLocation(oldProps.source, nextProps.source)
  );
}

function compareLocation(
  oldLocation: LocationSpecification,
  nextLocation: LocationSpecification,
): boolean {
  return (
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
  const oldChildren = getLocationChildren(oldLocation);
  const nextChildren = getLocationChildren(nextLocation);

  // Children shouldn't be checked if the row is collapsed
  const nextExpanded = 'expanded' in nextLocation && nextLocation.expanded;

  return (
    (!nextExpanded && oldChildren.length > 0 && nextChildren.length > 0) ||
    (oldChildren.length === nextChildren.length &&
      oldChildren.every((oldChild, i) => compareLocation(oldChild, nextChildren[i])))
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
