import * as React from 'react';
import { Animated, Component, GestureView, Styles, Types, View } from 'reactxp';
import { colors } from '../../config.json';

interface IProps {
  defaultOn: boolean;
  onChange?: (isOn: boolean) => void;
}

interface IState {
  isOn: boolean;
  isPressed: boolean;
}

const styles = {
  holder: Styles.createViewStyle({
    width: 52,
    height: 32,
    borderColor: colors.white,
    borderWidth: 2,
    borderStyle: 'solid',
    borderRadius: 16,
    padding: 2,
  }),
  knob: {
    base: Styles.createViewStyle({
      height: 24,
      borderRadius: 24,
      backgroundColor: colors.red,
    }),
    on: Styles.createViewStyle({
      backgroundColor: colors.green,
    }),
  },
};

interface IPosition { x: number; y: number }

const SWITCH_DEFAULT_WIDTH = 24;
const SWITCH_PRESSED_WIDTH = 28;

export default class Switch extends Component<IProps, IState> {
  public static defaultProps: Partial<IProps> = {
    defaultOn: false,
    onChange: undefined,
  };

  public state: IState = {
    isOn: false,
    isPressed: false,
  };

  private isPanning = false;
  private startPos = { x: 0, y: 0 };
  private startValue = false;

  private translationValue = Animated.createValue(0);
  private widthValue = Animated.createValue(SWITCH_DEFAULT_WIDTH);
  private animatedStyle = Styles.createAnimatedViewStyle({
    width: this.widthValue,
    transform: [
      {
        translateX: this.translationValue,
      },
    ],
  });
  private animation?: Types.Animated.CompositeAnimation;

  private knobRef = React.createRef<View>();

  constructor(props: IProps) {
    super(props);

    this.state.isOn = props.defaultOn;

    if (props.defaultOn) {
      this.translationValue.setValue(this.computeTranslation(props.defaultOn, false));
    }
  }

  public setOn(isOn: boolean) {
    this.isPanning = false;

    this.setState({ isOn, isPressed: false });
  }

  public shouldComponentUpdate(_nextProps: IProps, nextState: IState) {
    return nextState.isOn !== this.state.isOn || nextState.isPressed !== this.state.isPressed;
  }

  public componentDidUpdate(_prevProps: IProps, prevState: IState) {
    if (prevState.isOn !== this.state.isOn || prevState.isPressed !== this.state.isPressed) {
      this.animate();
    }
  }

  public render() {
    return (
      <GestureView
        preferredPan={Types.PreferredPanGesture.Horizontal}
        onPanHorizontal={this.onPanHorizontal}
        onTap={this.onTap}>
        <View style={styles.holder}>
          <Animated.View style={this.animatedStyle}>
            <View
              ref={this.knobRef}
              style={[styles.knob.base, this.state.isOn ? styles.knob.on : undefined]}
            />
          </Animated.View>
        </View>
      </GestureView>
    );
  }

  private onTap = (_gesture: Types.TapGestureState) => {
    this.setState(
      (state) => ({ isOn: !state.isOn, isPressed: false }),
      () => {
        this.notify();
      },
    );
  };

  private onPanHorizontal = (gesture: Types.PanGestureState) => {
    if (this.isPanning) {
      if (gesture.isComplete) {
        this.isPanning = false;

        this.setState({ isPressed: false }, () => {
          if (this.startValue !== this.state.isOn) {
            this.notify();
          }
        });
      } else {
        const currentPos = { x: gesture.clientX, y: gesture.clientY };
        const nextOn = this.computeNextState(this.startPos, currentPos);

        if (this.state.isOn !== nextOn) {
          this.startPos = currentPos;

          this.setState({ isOn: nextOn });
        }
      }
    } else {
      if (gesture.isComplete) {
        return;
      }

      this.isPanning = true;
      this.startPos = { x: gesture.clientX, y: gesture.clientY };
      this.startValue = this.state.isOn;
      this.setState({ isPressed: true });
    }
  };

  private computeNextState(initialPos: IPosition, currentPos: IPosition): boolean {
    if (currentPos.x < initialPos.x && this.state.isOn) {
      return false;
    } else if (currentPos.x > initialPos.x && !this.state.isOn) {
      return true;
    } else {
      return this.state.isOn;
    }
  }

  private computeKnobWidth(isPressed: boolean) {
    return isPressed ? SWITCH_PRESSED_WIDTH : SWITCH_DEFAULT_WIDTH;
  }

  private computeTranslation(isOn: boolean, isPressed: boolean) {
    if (isOn) {
      return isPressed ? 16 : 20;
    } else {
      return 0;
    }
  }

  private animate(onFinish?: (done: boolean) => void) {
    const duration = 200;
    const animation = Animated.parallel([
      Animated.timing(this.translationValue, {
        toValue: this.computeTranslation(this.state.isOn, this.state.isPressed),
        duration,
      }),
      Animated.timing(this.widthValue, {
        toValue: this.computeKnobWidth(this.state.isPressed),
        duration,
      }),
    ]);

    if (this.animation) {
      this.animation.stop();
    }

    animation.start((options) => {
      if (options.finished) {
        this.animation = undefined;
      }

      if (onFinish) {
        onFinish(options.finished);
      }
    });

    this.animation = animation;
  }

  private notify() {
    if (this.props.onChange) {
      this.props.onChange(this.state.isOn);
    }
  }
}
