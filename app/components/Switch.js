// @flow
import React, { Component } from 'react';

import type { Point2d } from '../types';

const CLICK_TIMEOUT = 1000;
const MOVE_THRESHOLD = 10;

export default class Switch extends Component {
  props: {
    isOn: boolean;
    onChange: ?((isOn: boolean) => void);
  }

  defaultProps = {
    isOn: false
  }

  ref: ?HTMLInputElement;
  onRef = (e: HTMLInputElement) => this.ref = e;

  state = {
    isTracking: false,
    ignoreChange: false,
    initialPos: (null: ?Point2d),
    startTime: (null: ?number)
  }

  handleMouseDown = (e: MouseEvent) => {
    const { pageX: x, pageY: y } = e;
    this.setState({
      isTracking: true,
      initialPos: { x, y },
      startTime: e.timeStamp
    });
  }

  handleMouseMove = (e: MouseEvent) => {
    if(!this.state.isTracking) {
      return;
    }

    const inputElement = this.ref;
    const { x: x0 } = this.state.initialPos;
    const { pageX: x, pageY: y } = e;
    const dx = Math.abs(x0 - x);

    if(dx < MOVE_THRESHOLD) {
      return;
    }

    const isOn = !!this.props.isOn;
    let nextOn = isOn;

    if(x < x0 && isOn) {
      nextOn = false;
    } else if(x > x0 && !isOn) {
      nextOn = true;
    }

    if(isOn !== nextOn) {
      this.setState({
        initialPos: { x, y },
        ignoreChange: true
      });

      if(inputElement) {
        inputElement.checked = nextOn;
      }

      this.notify(nextOn);
    }
  }

  handleMouseUp = () => {
    if(this.state.isTracking) {
      this.setState({
        isTracking: false,
        initialPos: null
      });
    }
  }

  handleChange = (e: Event) => {
    const startTime = this.state.startTime;
    const eventTarget = e.target;

    if(typeof(startTime) !== 'number') {
      throw new Error('startTime must be a number.');
    }

    if(!(eventTarget instanceof HTMLInputElement)) {
      throw new Error('e.target must be an instance of HTMLInputElement.');
    }

    const dt = e.timeStamp - startTime;

    if(this.state.ignoreChange) {
      this.setState({ ignoreChange: false });
      e.preventDefault();
    } else if(dt > CLICK_TIMEOUT) {
      e.preventDefault();
    } else {
      this.notify(eventTarget.checked);
    }
  }

  notify(isOn: boolean) {
    const onChange = this.props.onChange;
    if(onChange) {
      onChange(isOn);
    }
  }

  componentDidMount() {
    document.addEventListener('mousemove', this.handleMouseMove);
    document.addEventListener('mouseup', this.handleMouseUp);
  }

  componentWillUnmount() {
    document.removeEventListener('mousemove', this.handleMouseMove);
    document.removeEventListener('mouseup', this.handleMouseUp);
  }

  render(): React.Element<*> {
    return (
      <input type="checkbox" ref={ this.onRef } className="switch" checked={ this.props.isOn }
        onMouseDown={ this.handleMouseDown }
        onChange={ this.handleChange } />
    );
  }
}
