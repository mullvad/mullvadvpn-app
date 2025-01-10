import React from 'react';

import { IScrollEvent } from './CustomScrollbars';

interface NavigationContainerProps {
  children?: React.ReactNode;
}

interface NavigationContainerState {
  showsBarTitle: boolean;
}

export const NavigationScrollContext = React.createContext<{
  showsBarTitle: boolean;
  onScroll: (event: IScrollEvent) => void;
}>({
  showsBarTitle: false,
  onScroll: () => {
    throw new Error('NavigationScrollContext provider is missing.');
  },
});

export class NavigationContainer extends React.Component<
  NavigationContainerProps,
  NavigationContainerState
> {
  state: NavigationContainerState = {
    showsBarTitle: false,
  };

  componentDidMount() {
    this.updateBarAppearance({ scrollLeft: 0, scrollTop: 0 });
  }

  render() {
    const { children } = this.props;
    const { showsBarTitle } = this.state;

    return (
      <NavigationScrollContext.Provider
        value={{
          showsBarTitle,
          onScroll: this.onScroll,
        }}>
        {children}
      </NavigationScrollContext.Provider>
    );
  }

  private onScroll = (event: IScrollEvent) => {
    this.updateBarAppearance(event);
  };

  private updateBarAppearance = ({ scrollTop }: IScrollEvent) => {
    // Show the bar title when user scrolls past page title
    const showsBarTitle = scrollTop > 20;

    if (this.state.showsBarTitle !== showsBarTitle) {
      this.setState({ showsBarTitle });
    }
  };
}
