import React, { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../shared/gettext';
import { Icon } from '../../lib/components';
import { useHistory } from '../../lib/history';
import { RoutePath } from '../../lib/routes';
import { useSelector } from '../../redux/store';
import * as Cell from '../cell';
import InfoButton from '../InfoButton';
import { SpecialLocationIndicator } from '../RelayStatusIndicator';
import {
  getButtonColor,
  StyledLocationRowButton,
  StyledLocationRowContainerWithMargin,
  StyledLocationRowLabel,
} from './LocationRowStyles';
import { SpecialBridgeLocationType, SpecialLocation } from './select-location-types';

interface SpecialLocationsProps<T> {
  source: Array<SpecialLocation<T>>;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: T) => void;
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

const StyledSpecialLocationInfoButton = styled(InfoButton)({
  width: '56px',
  height: '48px',
  borderRadius: 0,
  '&:focus-visible': {
    zIndex: 10,
  },
});

interface SpecialLocationRowProps<T> {
  source: SpecialLocation<T>;
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: T) => void;
}

function SpecialLocationRow<T>(props: SpecialLocationRowProps<T>) {
  const { onSelect: propsOnSelect } = props;
  const onSelect = useCallback(() => {
    if (!props.source.selected) {
      propsOnSelect(props.source.value);
    }
  }, [props.source, propsOnSelect]);

  const innerProps: SpecialLocationRowInnerProps<T> = {
    ...props,
    onSelect,
  };
  return <props.source.component {...innerProps} />;
}

export interface SpecialLocationRowInnerProps<T>
  extends Omit<SpecialLocationRowProps<T>, 'onSelect'> {
  onSelect: () => void;
}

export function AutomaticLocationRow(
  props: SpecialLocationRowInnerProps<SpecialBridgeLocationType>,
) {
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  const background = getButtonColor(props.source.selected, 0, props.source.disabled);
  return (
    <StyledLocationRowContainerWithMargin ref={selectedRef}>
      <StyledLocationRowButton onClick={props.onSelect} $level={0} {...background}>
        <SpecialLocationIndicator />
        <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
      </StyledLocationRowButton>
      <Cell.SideButton
        as={StyledSpecialLocationInfoButton}
        title={messages.gettext('Automatic')}
        message={messages.pgettext(
          'select-location-view',
          'The app selects a random bridge server, but servers have a higher probability the closer they are to you.',
        )}
        aria-label={messages.pgettext('accessibility', 'info')}
        {...background}
      />
    </StyledLocationRowContainerWithMargin>
  );
}

export function CustomExitLocationRow(props: SpecialLocationRowInnerProps<undefined>) {
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  const background = getButtonColor(props.source.selected, 0, props.source.disabled);
  return (
    <StyledLocationRowContainerWithMargin ref={selectedRef}>
      <StyledLocationRowButton $level={0} {...background}>
        <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
      </StyledLocationRowButton>
    </StyledLocationRowContainerWithMargin>
  );
}

export function CustomBridgeLocationRow(
  props: SpecialLocationRowInnerProps<SpecialBridgeLocationType>,
) {
  const { push } = useHistory();

  const bridgeSettings = useSelector((state) => state.settings.bridgeSettings);
  const bridgeConfigured = bridgeSettings.custom !== undefined;
  const icon = bridgeConfigured ? 'edit-circle' : 'add-circle';

  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  const background = getButtonColor(props.source.selected, 0, props.source.disabled);

  const navigate = useCallback(() => push(RoutePath.editCustomBridge), [push]);

  return (
    <StyledLocationRowContainerWithMargin ref={selectedRef} disabled={props.source.disabled}>
      <StyledLocationRowButton
        as="button"
        onClick={props.onSelect}
        $level={0}
        disabled={props.source.disabled}
        {...background}>
        <SpecialLocationIndicator />
        <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
      </StyledLocationRowButton>
      <Cell.SideButton
        as={StyledSpecialLocationInfoButton}
        title={messages.pgettext('select-location-view', 'Custom bridge')}
        message={messages.pgettext(
          'select-location-view',
          'A custom bridge server can be used to circumvent censorship when regular Mullvad bridge servers don’t work.',
        )}
        $noSeparator
        {...background}
      />
      <Cell.SideButton
        {...background}
        aria-label={
          bridgeConfigured
            ? messages.pgettext('accessibility', 'Edit custom bridge')
            : messages.pgettext('accessibility', 'Add new custom bridge')
        }
        onClick={navigate}>
        <Icon icon={icon} />
      </Cell.SideButton>
    </StyledLocationRowContainerWithMargin>
  );
}
