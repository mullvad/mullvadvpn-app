import * as React from 'react';
import ReactDOM from 'react-dom';
import { Component, Styles, View } from 'reactxp';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
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
import Selector, { ISelectorItem, SelectorCell } from './Selector';
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

const styles = {
  page: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    flex: 1,
  }),
  container: Styles.createViewStyle({
    flex: 1,
  }),
  selector: Styles.createViewStyle({
    marginBottom: 0,
  }),
  // plain CSS style
  scrollview: {
    flex: 1,
  },
};

export default class SelectLanguage extends Component<IProps, IState> {
  private scrollView = React.createRef<CustomScrollbars>();
  private selectedCellRef = React.createRef<SelectorCell<string>>();

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
        <Container>
          <View style={styles.page}>
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

              <View style={styles.container}>
                <NavigationScrollbars style={styles.scrollview}>
                  <SettingsHeader>
                    <HeaderTitle>
                      {messages.pgettext('select-language-nav', 'Select language')}
                    </HeaderTitle>
                  </SettingsHeader>
                  <Selector
                    style={styles.selector}
                    title=""
                    values={this.state.source}
                    value={this.props.preferredLocale}
                    onSelect={this.props.setPreferredLocale}
                    selectedCellRef={this.selectedCellRef}
                  />
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  private scrollToSelectedCell() {
    const ref = this.selectedCellRef.current;
    const scrollView = this.scrollView.current;

    if (scrollView && ref) {
      const cellDOMNode = ReactDOM.findDOMNode(ref);
      if (cellDOMNode instanceof HTMLElement) {
        scrollView.scrollToElement(cellDOMNode, 'middle');
      }
    }
  }
}
