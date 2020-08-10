import * as React from 'react';
import { Component } from 'reactxp';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { compareRelayLocation, RelayLocation } from '../../shared/daemon-rpc-types';
import * as Cell from './Cell';
import RelayStatusIndicator from './RelayStatusIndicator';

interface IProps {
  location: RelayLocation;
  active: boolean;
  hostname: string;
  selected: boolean;
  onSelect?: (location: RelayLocation) => void;
}

const Button = styled(Cell.CellButton)((props: { selected: boolean }) => ({
  paddingRight: 0,
  paddingLeft: '50px',
  backgroundColor: !props.selected ? colors.blue20 : undefined,
}));

const Label = styled(Cell.Label)({
  '[disabled] &': {
    color: colors.white20,
  },
});

export default class RelayRow extends Component<IProps> {
  public static compareProps(oldProps: IProps, nextProps: IProps) {
    return (
      oldProps.hostname === nextProps.hostname &&
      oldProps.selected === nextProps.selected &&
      oldProps.active === nextProps.active &&
      compareRelayLocation(oldProps.location, nextProps.location)
    );
  }

  public shouldComponentUpdate(nextProps: IProps) {
    return !RelayRow.compareProps(this.props, nextProps);
  }

  public render() {
    return (
      <Button
        onClick={this.handleClick}
        selected={this.props.selected}
        disabled={!this.props.active}>
        <RelayStatusIndicator active={this.props.active} selected={this.props.selected} />

        <Label>{this.props.hostname}</Label>
      </Button>
    );
  }

  private handleClick = () => {
    if (this.props.onSelect) {
      this.props.onSelect(this.props.location);
    }
  };
}
