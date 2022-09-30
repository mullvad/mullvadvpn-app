import React, { useCallback, useRef } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { compareRelayLocation, RelayLocation } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import Accordion from './Accordion';
import * as Cell from './cell';
import ChevronButton from './ChevronButton';
import { normalText } from './common-styles';
import RelayStatusIndicator from './RelayStatusIndicator';

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
  paddingRight: '22px',

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

interface IProps {
  name: string;
  active: boolean;
  disabled: boolean;
  location: RelayLocation;
  selected: boolean;
  expanded?: boolean;
  onSelect?: (location: RelayLocation) => void;
  onExpand?: (location: RelayLocation, value: boolean) => void;
  onWillExpand?: (locationRect: DOMRect, expandedContentHeight: number) => void;
  onTransitionEnd?: () => void;
  children?: React.ReactElement<IProps>[];
}

function LocationRow(props: IProps, ref: React.Ref<HTMLDivElement>) {
  const hasChildren = props.children !== undefined;
  const buttonRef = useRef<HTMLButtonElement>() as React.RefObject<HTMLButtonElement>;

  const toggleCollapse = useCallback(() => {
    props.onExpand?.(props.location, !props.expanded);
  }, [props.onExpand, props.expanded, props.location]);

  const handleClick = useCallback(() => props.onSelect?.(props.location), [
    props.onSelect,
    props.location,
  ]);

  const onWillExpand = useCallback(
    (nextHeight: number) => {
      const buttonRect = buttonRef.current?.getBoundingClientRect();
      if (buttonRect) {
        props.onWillExpand?.(buttonRect, nextHeight);
      }
    },
    [props.onWillExpand],
  );

  return (
    <>
      <StyledLocationRowContainer ref={ref} disabled={props.disabled}>
        <StyledLocationRowButton
          as="button"
          ref={buttonRef}
          onClick={handleClick}
          selected={props.selected}
          location={props.location}
          disabled={props.disabled}>
          <RelayStatusIndicator active={props.active} selected={props.selected} />
          <StyledLocationRowLabel>{props.name}</StyledLocationRowLabel>
        </StyledLocationRowButton>
        {hasChildren ? (
          <StyledLocationRowIcon
            as={ChevronButton}
            onClick={toggleCollapse}
            up={props.expanded ?? false}
            selected={props.selected}
            disabled={props.disabled}
            location={props.location}
            aria-label={sprintf(
              props.expanded
                ? messages.pgettext('accessibility', 'Collapse %(location)s')
                : messages.pgettext('accessibility', 'Expand %(location)s'),
              { location: props.name },
            )}
          />
        ) : null}
      </StyledLocationRowContainer>

      {hasChildren && (
        <Accordion
          expanded={props.expanded}
          onWillExpand={onWillExpand}
          onTransitionEnd={props.onTransitionEnd}
          animationDuration={150}>
          <Cell.Group noMarginBottom>{props.children}</Cell.Group>
        </Accordion>
      )}
    </>
  );
}

export default React.memo(React.forwardRef(LocationRow), compareProps);

function compareProps(oldProps: IProps, nextProps: IProps): boolean {
  return (
    React.Children.count(oldProps.children) === React.Children.count(nextProps.children) &&
    oldProps.name === nextProps.name &&
    oldProps.active === nextProps.active &&
    oldProps.disabled === nextProps.disabled &&
    oldProps.selected === nextProps.selected &&
    oldProps.expanded === nextProps.expanded &&
    oldProps.onSelect === nextProps.onSelect &&
    oldProps.onExpand === nextProps.onExpand &&
    oldProps.onWillExpand === nextProps.onWillExpand &&
    oldProps.onTransitionEnd === nextProps.onTransitionEnd &&
    compareRelayLocation(oldProps.location, nextProps.location) &&
    compareChildren(oldProps.children, nextProps.children)
  );
}

function compareChildren(
  oldChildren?: React.ReactElement<IProps>[],
  nextChildren?: React.ReactElement<IProps>[],
) {
  if (oldChildren === undefined || nextChildren === undefined) {
    return oldChildren === nextChildren;
  }

  return (
    oldChildren.length === nextChildren.length &&
    oldChildren.every((oldChild, i) => compareProps(oldChild.props, nextChildren[i].props))
  );
}
