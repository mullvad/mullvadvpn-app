import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { AriaInputGroup } from './AriaGroup';
import CustomScrollbars from './CustomScrollbars';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import Selector, { ISelectorItem } from './Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

interface IProps {
  preferredLocale: string;
  preferredLocalesList: Array<{ name: string; code: string }>;
  setPreferredLocale: (locale: string) => void;
  onClose: () => void;
}

interface IState {
  source: Array<ISelectorItem<string>>;
}

const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

const StyledSelector = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

export default class SelectLanguage extends React.Component<IProps, IState> {
  private scrollView = React.createRef<CustomScrollbars>();
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
      <Layout>
        <StyledContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <BackBarItem action={this.props.onClose}>
                  {
                    // TRANSLATORS: Back button in navigation bar
                    messages.pgettext('navigation-bar', 'Settings')
                  }
                </BackBarItem>
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
                  values={this.state.source}
                  value={this.props.preferredLocale}
                  onSelect={this.props.setPreferredLocale}
                  selectedCellRef={this.selectedCellRef}
                />
              </AriaInputGroup>
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </StyledContainer>
      </Layout>
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
