import React, { Component, PropTypes } from 'react';

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
      startTime: null,
      target: null
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
    if(!this.state.isTracking) {
      return;
    }

    const thresholdX = 10, thresholdY = 50;
    const { x: x0, y: y0 } = this.state.initialPos;
    const { pageX: x, pageY: y } = e;

    const dx = Math.abs(x0 - x);
    const dy = Math.abs(y0 - y);

    if(dx < thresholdX || dy > thresholdY) {
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
      this.setState({ initialPos: { x, y } });
      this.refs.input.checked = nextOn;
      this.notify(nextOn);
      this.setState({ ignoreChange: true });
    }
  }

  handleMouseUp() {
    if(this.state.isTracking) {
      this.setState({ isTracking: false, initialPos: null });
      console.log('mouseup');
    }
  }

  handleChange(e) {
    console.log('ONCHANGE ' + e.target.checked);
    const delta = e.timeStamp - this.state.startTime;
    const threshold = 1000;

    if(this.state.ignoreChange) {
      e.preventDefault();
      this.setState({ ignoreChange: false });
    } else if(delta > threshold) {
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