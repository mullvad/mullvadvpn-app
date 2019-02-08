import * as React from 'react';

const CLICK_TIMEOUT = 1000;
const MOVE_THRESHOLD = 10;

interface IProps {
  className?: string;
  isOn: boolean;
  onChange?: (isOn: boolean) => void;
}

interface IState {
  ignoreChange: boolean;
  initialPos: { x: number; y: number };
  startTime?: number;
}

export default class Switch extends React.Component<IProps, IState> {
  public static defaultProps: Partial<IProps> = {
    isOn: false,
    onChange: undefined,
  };

  public state: IState = {
    ignoreChange: false,
    initialPos: { x: 0, y: 0 },
    startTime: undefined,
  };

  public isCapturingMouseEvents = false;
  public ref = React.createRef<HTMLInputElement>();

  public componentWillUnmount() {
    // guard from abrupt programmatic unmount
    if (this.isCapturingMouseEvents) {
      this.stopCapturingMouseEvents();
    }
  }

  public render() {
    const { isOn, onChange, ...otherProps } = this.props;
    const className = ('switch ' + (otherProps.className || '')).trim();
    return (
      <input
        {...otherProps}
        type="checkbox"
        ref={this.ref}
        className={className}
        checked={isOn}
        onMouseDown={this.handleMouseDown}
        onChange={this.handleChange}
      />
    );
  }

  private handleMouseDown = (e: React.MouseEvent<HTMLInputElement>) => {
    const { clientX: x, clientY: y } = e;
    this.startCapturingMouseEvents();
    this.setState({
      initialPos: { x, y },
      startTime: e.timeStamp,
    });
  };

  private handleMouseMove = (e: MouseEvent) => {
    const inputElement = this.ref.current;
    const { x: x0 } = this.state.initialPos;
    const { clientX: x, clientY: y } = e;
    const dx = Math.abs(x0 - x);

    if (dx < MOVE_THRESHOLD) {
      return;
    }

    const isOn = !!this.props.isOn;
    let nextOn = isOn;

    if (x < x0 && isOn) {
      nextOn = false;
    } else if (x > x0 && !isOn) {
      nextOn = true;
    }

    if (isOn !== nextOn) {
      this.setState({
        initialPos: { x, y },
        ignoreChange: true,
      });

      if (inputElement) {
        inputElement.checked = nextOn;
      }

      this.notify(nextOn);
    }
  };

  private handleMouseUp = () => {
    this.stopCapturingMouseEvents();
  };

  private handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const startTime = this.state.startTime;
    const eventTarget = e.target;

    if (typeof startTime !== 'number') {
      throw new Error('startTime must be a number.');
    }

    const dt = e.timeStamp - startTime;

    if (this.state.ignoreChange) {
      this.setState({ ignoreChange: false });
      e.preventDefault();
    } else if (dt > CLICK_TIMEOUT) {
      e.preventDefault();
    } else {
      this.notify(eventTarget.checked);
    }
  };

  private notify(isOn: boolean) {
    const onChange = this.props.onChange;
    if (onChange) {
      onChange(isOn);
    }
  }

  private startCapturingMouseEvents() {
    if (this.isCapturingMouseEvents) {
      throw new Error('startCapturingMouseEvents() is called out of order.');
    }
    document.addEventListener('mousemove', this.handleMouseMove);
    document.addEventListener('mouseup', this.handleMouseUp);
    this.isCapturingMouseEvents = true;
  }

  private stopCapturingMouseEvents() {
    if (!this.isCapturingMouseEvents) {
      throw new Error('stopCapturingMouseEvents() is called out of order.');
    }
    document.removeEventListener('mousemove', this.handleMouseMove);
    document.removeEventListener('mouseup', this.handleMouseUp);
    this.isCapturingMouseEvents = false;
  }
}
