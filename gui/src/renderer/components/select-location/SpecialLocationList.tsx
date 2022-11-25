import React, { useCallback } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { messages } from '../../../shared/gettext';
import * as Cell from '../cell';
import InfoButton from '../InfoButton';
import {
  StyledLocationRowButton,
  StyledLocationRowContainer,
  StyledLocationRowIcon,
  StyledLocationRowLabel,
} from './LocationRow';
import { LocationSelection, LocationSelectionType, SpecialLocation } from './select-location-types';

interface SpecialLocationsProps<T> {
  source: Array<SpecialLocation<T>>;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: LocationSelection<T>) => void;
}

export default function SpecialLocationList<T>({ source, ...props }: SpecialLocationsProps<T>) {
  return (
    <>
      {source.map((location) => (
        <SpecialLocationRow key={location.label} source={location} {...props} />
      ))}
    </>
  );
}

const StyledLocationRowContainerWithMargin = styled(StyledLocationRowContainer)({
  marginBottom: 1,
});

const StyledSpecialLocationIcon = styled(Cell.Icon)({
  flex: 0,
  marginLeft: '2px',
  marginRight: '8px',
});

const StyledSpecialLocationInfoButton = styled(InfoButton)({
  margin: 0,
  padding: '0 25px',
  backgroundColor: colors.blue,
});

interface SpecialLocationRowProps<T> {
  source: SpecialLocation<T>;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: LocationSelection<T>) => void;
}

function SpecialLocationRow<T>(props: SpecialLocationRowProps<T>) {
  const onSelect = useCallback(() => {
    if (!props.source.selected) {
      props.onSelect({
        type: LocationSelectionType.special,
        value: props.source.value,
      });
    }
  }, [props.source.selected, props.onSelect, props.source.value]);

  const icon = props.source.selected ? 'icon-tick' : props.source.icon ?? undefined;
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  return (
    <StyledLocationRowContainerWithMargin ref={selectedRef}>
      <StyledLocationRowButton onClick={onSelect} selected={props.source.selected}>
        {icon && (
          <StyledSpecialLocationIcon
            source={icon}
            tintColor={colors.white}
            height={22}
            width={22}
          />
        )}
        <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
      </StyledLocationRowButton>
      {props.source.info && (
        <StyledLocationRowIcon
          as={StyledSpecialLocationInfoButton}
          message={props.source.info}
          selected={props.source.selected}
          aria-label={messages.pgettext('accessibility', 'info')}
        />
      )}
    </StyledLocationRowContainerWithMargin>
  );
}
