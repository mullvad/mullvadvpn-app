// @flow
import React, { Component } from 'react';

type ScrollbarUpdateContext = {
  size: boolean,
  position: boolean,
};

export default class CustomScrollbars extends Component {
  props: {
    thumbInset: { x: number, y: number },
    children: ?React.Element<*>,
  };

  static defaultProps = {
    thumbInset: { x: 2, y: 2 },
  };

  _scrollableElement: ?HTMLElement;
  _thumbElement: ?HTMLElement;

  componentDidMount() {
    this._updateScrollbarsHelper({
      position: true,
      size: true
    });
  }

  componentDidUpdate() {
    this._updateScrollbarsHelper({
      position: true,
      size: true
    });
  }

  render() {
    return (
      <div className="custom-scrollbars">
        <div className="custom-scrollbars__thumb"
          style={{ position: 'absolute', top: 0, right: 0 }}
          ref={ this._onThumbRef }></div>
        <div className="custom-scrollbars__scrollable"
          style={{ overflow: 'auto' }}
          onScroll={ this._onScroll }
          ref={ this._onScrollableRef }>
          { this.props.children }
        </div>
      </div>
    );
  }


  _onScrollableRef = (ref) => {
    this._scrollableElement = ref;
  }

  _onThumbRef = (ref) => {
    this._thumbElement = ref;
  }

  _onScroll = () => {
    this._updateScrollbarsHelper({ position: true });
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

    const thumbHeight = thumb.clientHeight;

    // calculate the thumb boundary to make sure that the visual appearance of
    // a thumb at lowest point matches the bottom of scrollable view
    const thumbBoundary = visibleHeight - thumbHeight - (this.props.thumbInset.y * 2);

    // calculate thumb position based on scroll progress and thumb boundary
    // adding vertical inset to adjust the thumb's appearance
    const thumbPosition = (thumbBoundary * scrollPosition) + this.props.thumbInset.y;

    return {
      x: -this.props.thumbInset.x,
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
    if(scrollable && thumb) {
      this._updateScrollbars(scrollable, thumb, updateFlags);
    }
  }

  _updateScrollbars(scrollable: HTMLElement, thumb: HTMLElement, context: $Shape<ScrollbarUpdateContext>) {
    if(context.size) {
      const thumbHeight = this._computeThumbHeight(scrollable);
      thumb.style.setProperty('height', thumbHeight + 'px');

      // hide thumb when there is nothing to scroll
      if(thumbHeight < scrollable.offsetHeight) {
        thumb.style.setProperty('opacity', '1');
      } else {
        thumb.style.setProperty('opacity', '0');
      }
    }

    if(context.position) {
      const { x, y } = this._computeThumbPosition(scrollable, thumb);
      thumb.style.setProperty('transform', `translate(${x}px, ${y}px)`);
    }
  }
}
