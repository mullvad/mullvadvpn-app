// @flow

import * as React from 'react';

type ScrollbarUpdateContext = {
  size: boolean,
  position: boolean,
};

const AUTOHIDE_TIMEOUT = 1000;

type Props = {
  autoHide: boolean,
  trackPadding: { x: number, y: number },
  children?: React.Node,
};

type State = {
  canScroll: boolean,
  showScrollIndicators: boolean,
  showTrack: boolean,
  isTrackHovered: boolean,
  isDragging: boolean,
  dragStart: {
    x: number,
    y: number,
  },
  isWide: boolean,
};

type ScrollPosition = 'top' | 'bottom' | 'middle';

export default class CustomScrollbars extends React.Component<Props, State> {
  static defaultProps = {
    autoHide: true,
    trackPadding: { x: 2, y: 2 },
  };

  state = {
    canScroll: false,
    showScrollIndicators: true,
    showTrack: false,
    isTrackHovered: false,
    isDragging: false,
    dragStart: { x: 0, y: 0 },
    isWide: false,
  };

  _scrollableElement: ?HTMLElement;
  _trackElement: ?HTMLElement;
  _thumbElement: ?HTMLElement;
  _autoHideTimer: ?TimeoutID;

  scrollTo(x: number, y: number) {
    const scrollable = this._scrollableElement;
    if (scrollable) {
      scrollable.scrollLeft = x;
      scrollable.scrollTop = y;
    }
  }

  scrollToElement(child: HTMLElement, scrollPosition: ScrollPosition) {
    const scrollable = this._scrollableElement;
    if (scrollable) {
      // throw if child is not a descendant of scroll view
      if (!scrollable.contains(child)) {
        throw new Error(
          'Cannot scroll to an element which is not a descendant of CustomScrollbars.',
        );
      }

      const scrollTop = this._computeScrollTop(scrollable, child, scrollPosition);
      this.scrollTo(0, scrollTop);
    }
  }

  componentDidMount() {
    this._updateScrollbarsHelper({
      position: true,
      size: true,
    });

    document.addEventListener('mousemove', this.handleMouseMove);
    document.addEventListener('mouseup', this.handleMouseUp);
    document.addEventListener('mousedown', this.handleMouseDown);

    // show scroll indicators briefly when mounted
    if (this.props.autoHide) {
      this._startAutoHide();
    }
  }

  componentWillUnmount() {
    this._stopAutoHide();

    document.removeEventListener('mousemove', this.handleMouseMove);
    document.removeEventListener('mouseup', this.handleMouseUp);
    document.removeEventListener('mousedown', this.handleMouseDown);
  }

  componentDidUpdate() {
    this._updateScrollbarsHelper({
      position: true,
      size: true,
    });
  }

  handleEnterTrack = () => {
    this._stopAutoHide();
    this.setState({
      isTrackHovered: true,
      showScrollIndicators: true,
      showTrack: true,
      isWide: true,
    });
  };

  handleLeaveTrack = () => {
    this.setState({
      isTrackHovered: false,
    });

    // do not hide the scrollbar if user is dragging a thumb but left the track area.
    if (this.props.autoHide && !this.state.isDragging) {
      this._startAutoHide();
    }
  };

  handleMouseDown = (event: MouseEvent) => {
    const thumb = this._thumbElement;
    const cursorPosition = {
      x: event.clientX,
      y: event.clientY,
    };

    // initiate dragging when user clicked inside of thumb
    if (thumb && this._isPointInsideOfElement(thumb, cursorPosition)) {
      this.setState({
        isDragging: true,
        dragStart: this._getPointRelativeToElement(thumb, cursorPosition),
      });
    }
  };

  handleMouseUp = (event: MouseEvent) => {
    if (!this.state.isDragging) {
      return;
    }

    this.setState({
      isDragging: false,
    });

    const track = this._trackElement;
    if (track) {
      // Make sure to auto-hide the scrollbar if cursor ended up outside of scroll track
      const cursorPosition = {
        x: event.clientX,
        y: event.clientY,
      };

      if (this.props.autoHide && !this._isPointInsideOfElement(track, cursorPosition)) {
        this._startAutoHide();
      }
    }
  };

  handleMouseMove = (event: MouseEvent) => {
    const scrollable = this._scrollableElement;
    const thumb = this._thumbElement;
    const track = this._trackElement;

    const cursorPosition = {
      x: event.clientX,
      y: event.clientY,
    };

    if (this.state.isDragging && scrollable && thumb) {
      // the content height of the scroll view
      const scrollHeight = scrollable.scrollHeight;

      // the visible height of the scroll view
      const visibleHeight = scrollable.offsetHeight;

      // lowest point of scrollTop
      const maxScrollTop = scrollHeight - visibleHeight;

      // Map absolute cursor coordinate to point in scroll container
      const pointInScrollContainer = this._getPointRelativeToElement(scrollable, cursorPosition);

      // calculate the thumb boundary to make sure that the visual appearance of
      // a thumb at the lowest point matches the bottom of scrollable view
      const thumbBoundary = this._computeTrackLength(scrollable) - thumb.clientHeight;
      const thumbTop = pointInScrollContainer.y - this.state.dragStart.y - this.props.trackPadding.y;
      const newScrollTop = (thumbTop / thumbBoundary) * maxScrollTop;

      scrollable.scrollTop = newScrollTop;
    }

    if (scrollable && track) {
      const intersectsTrack = this._isPointInsideOfElement(track, cursorPosition);

      if (!this.state.isTrackHovered && intersectsTrack) {
        this.handleEnterTrack();
      } else if (this.state.isTrackHovered && !intersectsTrack) {
        this.handleLeaveTrack();
      }
    }
  };

  render() {
    const { autoHide: _autoHide, trackPadding: _trackPadding, children, ...otherProps } = this.props;
    const showScrollbars = this.state.canScroll && this.state.showScrollIndicators;
    const thumbAnimationClass = showScrollbars ? ' custom-scrollbars__thumb--visible' : '';
    const thumbActiveClass =
      this.state.isTrackHovered || this.state.isDragging ? ' custom-scrollbars__thumb--active' : '';
    const thumbWideClass = this.state.isWide ? ' custom-scrollbars__thumb--wide' : '';
    const trackClass =
      showScrollbars && this.state.showTrack ? ' custom-scrollbars__track--visible' : '';

    return (
      <div {...otherProps} className="custom-scrollbars">
        <div className={`custom-scrollbars__track ${trackClass}`} ref={this._onTrackRef} />
        <div
          className={`custom-scrollbars__thumb ${thumbWideClass} ${thumbActiveClass} ${thumbAnimationClass}`}
          style={{ position: 'absolute', top: 0, right: 0 }}
          ref={this._onThumbRef}
        />
        <div
          className="custom-scrollbars__scrollable"
          style={{ overflow: 'auto' }}
          onScroll={this._onScroll}
          ref={this._onScrollableRef}>
          {children}
        </div>
      </div>
    );
  }

  _onScrollableRef = (ref) => {
    this._scrollableElement = ref;
  };

  _onTrackRef = (ref) => {
    this._trackElement = ref;
  };

  _onThumbRef = (ref) => {
    this._thumbElement = ref;
  };

  _onScroll = () => {
    this._updateScrollbarsHelper({ position: true });

    if (this.props.autoHide) {
      this._ensureScrollbarsVisible();

      // only auto-hide when scrolling with mousewheel
      if (!this.state.isDragging) {
        this._startAutoHide();
      }
    }
  };

  _ensureScrollbarsVisible() {
    if (!this.state.showScrollIndicators) {
      this.setState({
        showScrollIndicators: true,
      });
    }
  }

  _startAutoHide() {
    if (this._autoHideTimer) {
      clearTimeout(this._autoHideTimer);
    }

    this._autoHideTimer = setTimeout(() => {
      this.setState({
        showScrollIndicators: false,
        showTrack: false,
        isWide: false,
      });
    }, AUTOHIDE_TIMEOUT);
  }

  _stopAutoHide() {
    if (this._autoHideTimer) {
      clearTimeout(this._autoHideTimer);
      this._autoHideTimer = null;
    }
  }

  _isPointInsideOfElement(element: HTMLElement, point: { x: number, y: number }) {
    const rect = element.getBoundingClientRect();
    return (
      point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom
    );
  }

  _getPointRelativeToElement(element: HTMLElement, point: { x: number, y: number }) {
    const rect = element.getBoundingClientRect();
    return {
      x: point.x - rect.left,
      y: point.y - rect.top,
    };
  }

  _computeTrackLength(scrollable: HTMLElement) {
    return scrollable.offsetHeight - this.props.trackPadding.y * 2;
  }

  // Computes the position of child element within scrollable container
  _computeOffsetTop(scrollable: HTMLElement, child: HTMLElement) {
    let offsetTop = 0;
    let node = child;

    while (node && scrollable.contains(node)) {
      offsetTop += node.offsetTop;

      // Flow bug in offsetParent definition:
      // https://github.com/facebook/flow/issues/4407
      node = ((node.offsetParent: any): HTMLElement);
    }

    return offsetTop;
  }

  _computeScrollTop(scrollable: HTMLElement, child: HTMLElement, scrollPosition: ScrollPosition) {
    const offsetTop = this._computeOffsetTop(scrollable, child);

    switch (scrollPosition) {
      case 'top':
        return offsetTop;

      case 'bottom':
        return offsetTop - (scrollable.offsetHeight - child.clientHeight);

      case 'middle':
        return offsetTop - (scrollable.offsetHeight - child.clientHeight) * 0.5;

      default:
        throw new Error(`Unknown enum type for ScrollPosition: ${(scrollPosition: empty)}`);
    }
  }

  _computeThumbPosition(scrollable: HTMLElement, thumb: HTMLElement) {
    // the content height of the scroll view
    const scrollHeight = scrollable.scrollHeight;

    // the visible height of the scroll view
    const visibleHeight = scrollable.offsetHeight;

    // scroll offset
    const scrollTop = scrollable.scrollTop;

    // lowest point of scrollTop
    const maxScrollTop = scrollHeight - visibleHeight;

    // calculate scroll position within 0..1 range
    const scrollPosition = scrollHeight > 0 ? scrollTop / maxScrollTop : 0;

    // calculate the thumb boundary to make sure that the visual appearance of
    // a thumb at the lowest point matches the bottom of scrollable view
    const thumbBoundary = this._computeTrackLength(scrollable) - thumb.clientHeight;

    // calculate thumb position based on scroll progress and thumb boundary
    // adding vertical inset to adjust the thumb's appearance
    const thumbPosition = thumbBoundary * scrollPosition + this.props.trackPadding.y;

    return {
      x: -this.props.trackPadding.x,
      y: thumbPosition,
    };
  }

  _computeThumbHeight(scrollable: HTMLElement) {
    const scrollHeight = scrollable.scrollHeight;
    const visibleHeight = scrollable.offsetHeight;

    const thumbHeight = (visibleHeight / scrollHeight) * visibleHeight;

    // ensure that the scroll thumb doesn't shrink to nano size
    return Math.max(thumbHeight, 8);
  }

  _updateScrollbarsHelper(updateFlags: $Shape<ScrollbarUpdateContext>) {
    const scrollable = this._scrollableElement;
    const thumb = this._thumbElement;
    if (scrollable && thumb) {
      this._updateScrollbars(scrollable, thumb, updateFlags);
    }
  }

  _updateScrollbars(
    scrollable: HTMLElement,
    thumb: HTMLElement,
    context: $Shape<ScrollbarUpdateContext>,
  ) {
    if (context.size) {
      const thumbHeight = this._computeThumbHeight(scrollable);
      thumb.style.setProperty('height', thumbHeight + 'px');

      // hide thumb when there is nothing to scroll
      const canScroll = thumbHeight < scrollable.offsetHeight;
      if (this.state.canScroll !== canScroll) {
        this.setState({ canScroll });

        // flash the scroll indicators when the view becomes scrollable
        if (this.props.autoHide && canScroll) {
          this._startAutoHide();
          this._ensureScrollbarsVisible();
        }
      }
    }

    if (context.position) {
      const { x, y } = this._computeThumbPosition(scrollable, thumb);
      thumb.style.setProperty('transform', `translate(${x}px, ${y}px)`);
    }
  }
}
