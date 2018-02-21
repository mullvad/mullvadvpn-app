// @flow
import * as React from 'react';

export type AccordionProps = {
  height?: number | string,
  transitionStyle?: string,
  children?: React.Node
};

type AccordionState = {
  computedHeight: ?number | ?string,
};

export default class Accordion extends React.Component<AccordionProps, AccordionState> {
  static defaultProps = {
    height: 'auto',
    transitionStyle: 'height 0.25s ease-in-out'
  };

  state = {
    computedHeight: null,
  };

  _containerElement: ?HTMLElement;
  _contentElement: ?HTMLElement;

  constructor(props: AccordionProps) {
    super(props);

    // set the initial height if it's known
    if(props.height !== 'auto') {
      this.state = {
        computedHeight: props.height
      };
    }
  }

  componentDidMount() {
    const containerElement = this._containerElement;
    if(!containerElement) {
      throw new Error('containerElement cannot be null');
    }
    containerElement.addEventListener('transitionend', this._onTransitionEnd);
  }

  componentWillUnmount() {
    const containerElement = this._containerElement;
    if(!containerElement) {
      throw new Error('containerElement cannot be null');
    }
    containerElement.removeEventListener('transitionend', this._onTransitionEnd);
  }

  componentDidUpdate(prevProps: AccordionProps, _prevState: AccordionState) {
    if(prevProps.height !== this.props.height) {
      (async () => {
        const { transitionStyle } = this.props;

        // make sure to warm up CSS transition before updating height
        // do not warm up transitions if they are not expected to run
        if(transitionStyle && transitionStyle.toLowerCase() !== 'none') {
          await this._warmupTransition();
          this._updateHeight();
        } else {
          this._updateHeight();
          this._onTransitionEnd();
        }

      })();
    }
  }

  render() {
    const { height: _height, children, transitionStyle, ...otherProps } = this.props;
    let style = {
      transition: transitionStyle,
    };

    if(typeof(this.state.computedHeight) === 'number') {
      style = {
        ...style,
        overflow: 'hidden',
        height: this.state.computedHeight.toString() + 'px',
      };
    }

    return (
      <div { ...otherProps } style={ style } ref={ this._onContainerRef }>
        <div ref={ this._onContentRef }>
          { children }
        </div>
      </div>
    );
  }

  // Sets initial height and delays transition until next runloop
  // to make sure CSS transitions properly kick in.
  // This method resolves immediately if the height is already set.
  _warmupTransition(): Promise<void> {
    const contentElement = this._contentElement;
    if(!contentElement) {
      throw new Error('contentElement cannot be null');
    }
    return new Promise((resolve, _) => {
      // CSS transition always needs the initial height
      // to perform the animation
      if(this.state.computedHeight === null) {
        this.setState({
          computedHeight: contentElement.clientHeight
        }, () => {
          // important to skip a run loop
          // for CSS transition to kick in
          setTimeout(resolve, 0);
        });
      } else {
        resolve();
      }
    });
  }

  _updateHeight() {
    const contentElement = this._contentElement;
    if(!contentElement) {
      throw new Error('contentElement cannot be null');
    }
    this.setState({
      computedHeight: this.props.height === 'auto' ?
        contentElement.clientHeight :
        this.props.height
    });
  }

  _onTransitionEnd = () => {
    // reset height after transition to let element layout naturally
    if(this.props.height === 'auto') {
      this.setState({
        computedHeight: null,
      });
    }
  }

  _onContainerRef = (element) => {
    this._containerElement = element;
  }

  _onContentRef = (element) => {
    this._contentElement = element;
  }
}