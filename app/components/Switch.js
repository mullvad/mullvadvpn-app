import React, { Component, PropTypes } from 'react';

const CLICK_TIMEOUT = 1000;
const MOVE_THRESHOLD = 10;

export default class Switch extends Component {

  static propTypes = {
    isOn: PropTypes.bool,
    onChange: PropTypes.func
  }

  constructor(props) {
    super(props);
    this.state = {
      isTracking: false,
      ignoreChange: false,
      initialPos: null,
      startTime: null
    };
  }

  handleMouseDown(e) {
    const { pageX: x, pageY: y } = e;
    this.setState({
      isTracking: true,
      initialPos: { x, y },
      startTime: e.timeStamp
    });
  }

  handleMouseMove(e) {
    if(!this.state.isTracking) { return; }

    const { x: x0 } = this.state.initialPos;
    const { pageX: x, pageY: y } = e;
    const dx = Math.abs(x0 - x);

    if(dx < MOVE_THRESHOLD) { return; }

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
      this.refs.input.checked = nextOn;
      this.notify(nextOn);
    }
  }

  handleMouseUp() {
    if(this.state.isTracking) {
      this.setState({
        isTracking: false,
        initialPos: null
      });
    }
  }

  handleChange(e) {
    const dt = e.timeStamp - this.state.startTime;

    if(this.state.ignoreChange) {
      this.setState({ ignoreChange: false });
      e.preventDefault();
    } else if(dt > CLICK_TIMEOUT) {
      e.preventDefault();
    } else {
      this.notify(e.target.checked);
    }
  }

  notify(isOn) {
    if(this.props.onChange) {
      this.props.onChange(isOn);
    }
  }

  componentDidMount() {
    document.addEventListener('mousemove', ::this.handleMouseMove);
    document.addEventListener('mouseup', ::this.handleMouseUp);
  }

  componentWillUnmount() {
    document.removeEventListener('mousemove', ::this.handleMouseMove);
    document.removeEventListener('mouseup', ::this.handleMouseUp);
  }

  render() {
    return (
      <input type="checkbox" ref="input" className="switch" checked={ this.props.isOn }
             onMouseDown={ ::this.handleMouseDown } onChange={ ::this.handleChange } />
    );
  }
}
