import React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';

interface IProps {
  isOn: boolean;
  onChange?: (isOn: boolean) => void;
  className?: string;
}

interface IState {
  isOn: boolean;
  isPressed: boolean;
}

const PAN_DISTANCE = 10;

const SwitchContainer = styled.div({
  position: 'relative',
  width: '52px',
  height: '32px',
  borderColor: colors.white,
  borderWidth: '2px',
  borderStyle: 'solid',
  borderRadius: '16px',
  padding: '2px',
});

const Knob = styled.div({}, (props: { isOn: boolean; isPressed: boolean }) => ({
  position: 'absolute',
  height: '24px',
  borderRadius: '12px',
  transition: 'all 200ms linear',
  width: props.isPressed ? '28px' : '24px',
  backgroundColor: props.isOn ? colors.green : colors.red,
  // When enabled the button should be placed all the way to the right (100%) minus padding (2px).
  left: props.isOn ? 'calc(100% - 2px)' : '2px',
  // This moves the knob to the left making the right side aligned with the parent's right side.
  transform: `translateX(${props.isOn ? '-100%' : '0'})`,
}));

export default class Switch extends React.Component<IProps, IState> {
  public state: IState = {
    isOn: this.props.isOn,
    isPressed: false,
  };

  private containerRef = React.createRef<HTMLDivElement>();

  private isPanning = false;
  private startPos = 0;
  private changedDuringPan = false;

  public shouldComponentUpdate(nextProps: IProps, nextState: IState) {
    return (
      nextState.isOn !== this.state.isOn ||
      nextState.isPressed !== this.state.isPressed ||
      nextProps.isOn !== this.props.isOn
    );
  }

  public componentDidUpdate(prevProps: IProps, _prevState: IState) {
    if (
      this.props.isOn !== prevProps.isOn &&
      this.props.isOn !== this.state.isOn &&
      !this.isPanning
    ) {
      this.setState({ isOn: this.props.isOn });
    }
  }

  public render() {
    return (
      <SwitchContainer
        ref={this.containerRef}
        onClick={this.handleClick}
        className={this.props.className}>
        <Knob
          isOn={this.state.isOn}
          isPressed={this.state.isPressed}
          onMouseDown={this.handleMouseDown}
        />
      </SwitchContainer>
    );
  }

  private handleClick = () => {
    if (!this.changedDuringPan) {
      this.setState((state) => ({ isOn: !state.isOn }), this.notify);
    }

    // Needs to be reset to allow clicks on container after panning.
    this.changedDuringPan = false;
  };

  private handleMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    this.isPanning = true;
    this.startPos = event.clientX;
    this.changedDuringPan = false;

    document.addEventListener('mouseup', this.handleMouseUp);
    document.addEventListener('mousemove', this.handleMouseMove);
  };

  private handleMouseUp = (event: MouseEvent) => {
    document.removeEventListener('mouseup', this.handleMouseUp);
    document.removeEventListener('mousemove', this.handleMouseMove);

    this.setState({ isPressed: false });
    this.isPanning = false;
    // Reset changedDuringPan when onClick wont be called.
    if (event.target instanceof Element && !this.containerRef.current?.contains(event.target)) {
      this.changedDuringPan = false;
    }

    if (this.props.isOn !== this.state.isOn) {
      this.notify();
    }
  };

  private handleMouseMove = (event: MouseEvent) => {
    if (this.isPanning) {
      this.setState({ isPressed: true });

      const nextOn = this.computeNextState(event.clientX);
      if (this.state.isOn !== nextOn) {
        this.startPos = event.clientX;
        this.changedDuringPan = true;
        this.setState({ isOn: nextOn });
      }
    }
  };

  private computeNextState(currentPos: number): boolean {
    if (currentPos + PAN_DISTANCE < this.startPos && this.state.isOn) {
      return false;
    } else if (currentPos - PAN_DISTANCE > this.startPos && !this.state.isOn) {
      return true;
    } else {
      return this.state.isOn;
    }
  }

  private notify() {
    this.props.onChange?.(this.state.isOn);
  }
}
