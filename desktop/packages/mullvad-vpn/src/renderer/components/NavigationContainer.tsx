import React from 'react';

import { IScrollEvent } from './CustomScrollbars';

interface NavigationContainerProps {
  children?: React.ReactNode;
}

interface NavigationContainerState {
  showsBarTitle: boolean;
  showsBarSeparator: boolean;
}

export class NavigationContainer extends React.Component<
  NavigationContainerProps,
  NavigationContainerState
> {
  public state = {
    showsBarTitle: false,
    showsBarSeparator: false,
  };

  public componentDidMount() {
    this.updateBarAppearance({ scrollLeft: 0, scrollTop: 0 });
  }

  public render() {
    return (
      <NavigationScrollContext.Provider
        value={{
          ...this.state,
          onScroll: this.onScroll,
        }}>
        {this.props.children}
      </NavigationScrollContext.Provider>
    );
  }

  public onScroll = (event: IScrollEvent) => {
    this.updateBarAppearance(event);
  };

  private updateBarAppearance(event: IScrollEvent) {
    // that's where SettingsHeader.HeaderTitle intersects the navigation bar
    const showsBarSeparator = event.scrollTop > 11;

    // that's when SettingsHeader.HeaderTitle goes behind the navigation bar
    const showsBarTitle = event.scrollTop > 20;

    if (
      this.state.showsBarSeparator !== showsBarSeparator ||
      this.state.showsBarTitle !== showsBarTitle
    ) {
      this.setState({ showsBarSeparator, showsBarTitle });
    }
  }
}
export const NavigationScrollContext = React.createContext({
  showsBarTitle: false,
  showsBarSeparator: false,
  onScroll(_event: IScrollEvent): void {
    throw Error('NavigationScrollContext provider missing');
  },
});
