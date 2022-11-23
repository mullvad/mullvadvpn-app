import React, { useCallback, useRef } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { compareRelayLocation, RelayLocation } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import Accordion from '../Accordion';
import * as Cell from '../cell';
import ChevronButton from '../ChevronButton';
import { measurements, normalText } from '../common-styles';
import RelayStatusIndicator from '../RelayStatusIndicator';
import {
  CitySpecification,
  CountrySpecification,
  getLocationChildren,
  LocationSelection,
  LocationSelectionType,
  LocationSpecification,
  RelaySpecification,
} from './select-location-types';

interface IButtonColorProps {
  selected: boolean;
  disabled?: boolean;
  location?: RelayLocation;
}

const buttonColor = (props: IButtonColorProps) => {
  let background = colors.blue;
  if (props.selected) {
    background = colors.green;
  } else if (props.location) {
    if ('hostname' in props.location) {
      background = colors.blue20;
    } else if ('city' in props.location) {
      background = colors.blue40;
    }
  }

  let backgroundHover = colors.blue80;
  if (props.selected || props.disabled) {
    backgroundHover = background;
  } else if (props.location) {
    backgroundHover = colors.blue80;
  }

  return {
    backgroundColor: background,
    ':not(:disabled):hover': {
      backgroundColor: backgroundHover,
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
  (props: { location?: RelayLocation }) => {
    const paddingLeft =
      props.location && 'hostname' in props.location
        ? 50
        : props.location && 'city' in props.location
        ? 34
        : 18;

    return {
      flex: 1,
      border: 'none',
      padding: `0 10px 0 ${paddingLeft}px`,
      margin: 0,
    };
  },
);

export const StyledLocationRowIcon = styled.button(buttonColor, {
  position: 'relative',
  alignSelf: 'stretch',
  paddingLeft: '22px',
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
  fontWeight: 400,
});

interface IProps<C extends LocationSpecification> {
  source: C;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: LocationSelection<never>) => void;
  onExpand: (location: RelayLocation) => void;
  onCollapse: (location: RelayLocation) => void;
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
      props.onSelect({ type: LocationSelectionType.relay, value: props.source.location });
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

  // The selectedRef should only be used if the element is selected
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  return (
    <>
      <StyledLocationRowContainer ref={selectedRef} disabled={props.source.disabled}>
        <StyledLocationRowButton
          as="button"
          ref={buttonRef}
          onClick={handleClick}
          selected={props.source.selected}
          location={props.source.location}
          disabled={props.source.disabled}>
          <RelayStatusIndicator active={props.source.active} selected={props.source.selected} />
          <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
        </StyledLocationRowButton>
        {hasChildren ? (
          <StyledLocationRowIcon
            as={ChevronButton}
            onClick={toggleCollapse}
            up={expanded ?? false}
            selected={props.source.selected}
            disabled={props.source.disabled}
            location={props.source.location}
            aria-label={sprintf(
              expanded === true
                ? messages.pgettext('accessibility', 'Collapse %(location)s')
                : messages.pgettext('accessibility', 'Expand %(location)s'),
              { location: props.source.label },
            )}
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
    !nextExpanded ||
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
