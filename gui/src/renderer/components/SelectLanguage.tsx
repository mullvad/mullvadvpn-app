import * as React from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { AriaInputGroup } from './AriaGroup';
import Selector, { SelectorItem } from './cell/Selector';
import { CustomScrollbarsRef } from './CustomScrollbars';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

interface IProps {
  preferredLocale: string;
  preferredLocalesList: Array<{ name: string; code: string }>;
  setPreferredLocale: (locale: string) => void;
  onClose: () => void;
}

interface IState {
  source: Array<SelectorItem<string>>;
}

const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

const StyledSelector = styled(Selector)({
  marginBottom: 0,
}) as typeof Selector;

export default class SelectLanguage extends React.Component<IProps, IState> {
  private scrollView = React.createRef<CustomScrollbarsRef>();
  private selectedCellRef = React.createRef<HTMLButtonElement>();

  constructor(props: IProps) {
    super(props);

    this.state = {
      source: [
        ...this.props.preferredLocalesList.map((item) => ({ label: item.name, value: item.code })),
      ],
    };
  }

  public componentDidMount() {
    this.scrollToSelectedCell();
  }

  public render() {
    return (
      <BackAction action={this.props.onClose}>
        <Layout>
          <SettingsContainer>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <TitleBarItem>
                    {
                      // TRANSLATORS: Title label in navigation bar
                      messages.pgettext('select-language-nav', 'Select language')
                    }
                  </TitleBarItem>
                </NavigationItems>
              </NavigationBar>

              <StyledNavigationScrollbars ref={this.scrollView}>
                <SettingsHeader>
                  <HeaderTitle>
                    {messages.pgettext('select-language-nav', 'Select language')}
                  </HeaderTitle>
                </SettingsHeader>
                <AriaInputGroup>
                  <StyledSelector
                    title=""
                    items={this.state.source}
                    value={this.props.preferredLocale}
                    onSelect={this.props.setPreferredLocale}
                    selectedCellRef={this.selectedCellRef}
                  />
                </AriaInputGroup>
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </SettingsContainer>
        </Layout>
      </BackAction>
    );
  }

  private scrollToSelectedCell() {
    const ref = this.selectedCellRef.current;
    const scrollView = this.scrollView.current;

    if (scrollView && ref) {
      if (ref instanceof HTMLElement) {
        scrollView.scrollToElement(ref, 'middle');
      }
    }
  }
}
