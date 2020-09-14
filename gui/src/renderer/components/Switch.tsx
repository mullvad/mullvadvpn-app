import React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';

interface IProps {
  id?: string;
  'aria-labelledby'?: string;
  'aria-describedby'?: string;
  isOn: boolean;
  onChange?: (isOn: boolean) => void;
  className?: string;
  disabled?: boolean;
}

interface IState {
  isOn: boolean;
  isPressed: boolean;
}

const PAN_DISTANCE = 10;

const SwitchContainer = styled.div({}, (props: { disabled: boolean }) => ({
  position: 'relative',
  width: '48px',
  height: '30px',
  borderColor: props.disabled ? colors.white20 : colors.white80,
  borderWidth: '2px',
  borderStyle: 'solid',
  borderRadius: '16px',
  padding: '2px',
}));

const Knob = styled.div({}, (props: { isOn: boolean; isPressed: boolean; disabled: boolean }) => {
  let backgroundColor = props.isOn ? colors.green : colors.red;
  if (props.disabled) {
    backgroundColor = props.isOn ? colors.green40 : colors.red40;
  }

  return {
    position: 'absolute',
    height: '22px',
    borderRadius: '11px',
    transition: 'all 200ms linear',
    width: props.isPressed ? '26px' : '22px',
    backgroundColor,
    // When enabled the button should be placed all the way to the right (100%) minus padding (2px).
    left: props.isOn ? 'calc(100% - 2px)' : '2px',
    // This moves the knob to the left making the right side aligned with the parent's right side.
    transform: `translateX(${props.isOn ? '-100%' : '0'})`,
  };
});

export default class Switch extends React.PureComponent<IProps, IState> {
  public state: IState = {
    isOn: this.props.isOn,
    isPressed: false,
  };

  private containerRef = React.createRef<HTMLDivElement>();

  private isPanning = false;
  private startPos = 0;
  private changedDuringPan = false;

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
        id={this.props.id}
        role="checkbox"
        aria-labelledby={this.props['aria-labelledby']}
        aria-describedby={this.props['aria-describedby']}
        aria-checked={this.props.isOn}
        ref={this.containerRef}
        onClick={this.handleClick}
        disabled={this.props.disabled ?? false}
        className={this.props.className}>
        <Knob
          disabled={this.props.disabled ?? false}
          isOn={this.state.isOn}
          isPressed={this.state.isPressed}
          onMouseDown={this.handleMouseDown}
        />
      </SwitchContainer>
    );
  }

  private handleClick = () => {
    if (this.props.disabled) {
      return;
    }

    if (!this.changedDuringPan) {
      this.setState((state) => ({ isOn: !state.isOn }), this.notify);
    }

    // Needs to be reset to allow clicks on container after panning.
    this.changedDuringPan = false;
  };

  private handleMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    if (this.props.disabled) {
      return;
    }

    this.isPanning = true;
    this.startPos = event.clientX;
    this.changedDuringPan = false;

    document.addEventListener('mouseup', this.handleMouseUp);
    document.addEventListener('mousemove', this.handleMouseMove);
  };

  private handleMouseUp = (event: MouseEvent) => {
    if (this.props.disabled) {
      return;
    }

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
    if (this.props.disabled) {
      return;
    }

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
