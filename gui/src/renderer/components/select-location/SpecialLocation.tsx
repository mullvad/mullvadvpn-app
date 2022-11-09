import React from 'react';
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
});

export enum SpecialLocationIcon {
  geoLocation = 'icon-nearest',
}

interface ISpecialLocationProps<T> {
  icon: SpecialLocationIcon;
  value: T;
  isSelected?: boolean;
  onSelect?: (value: T) => void;
  info?: string;
  forwardedRef?: React.Ref<HTMLButtonElement>;
  children?: React.ReactNode;
}

export class SpecialLocation<T> extends React.Component<ISpecialLocationProps<T>> {
  public render() {
    return (
      <StyledLocationRowContainerWithMargin>
        <StyledLocationRowButton onClick={this.onSelect} selected={this.props.isSelected ?? false}>
          <StyledSpecialLocationIcon
            source={this.props.isSelected ? 'icon-tick' : this.props.icon}
            tintColor={colors.white}
            height={22}
            width={22}
          />
          <StyledLocationRowLabel>{this.props.children}</StyledLocationRowLabel>
        </StyledLocationRowButton>
        <StyledLocationRowIcon
          as={StyledSpecialLocationInfoButton}
          message={this.props.info}
          selected={this.props.isSelected ?? false}
          aria-label={messages.pgettext('accessibility', 'info')}
        />
      </StyledLocationRowContainerWithMargin>
    );
  }

  private onSelect = () => {
    if (!this.props.isSelected && this.props.onSelect) {
      this.props.onSelect(this.props.value);
    }
  };
}
