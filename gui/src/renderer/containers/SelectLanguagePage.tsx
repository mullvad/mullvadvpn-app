import { goBack } from 'connected-react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLanguage from '../components/SelectLanguage';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => ({
  preferredLocale: state.settings.guiSettings.preferredLocale,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);

  return {
    preferredLocalesList: props.app.getPreferredLocaleList(),
    setPreferredLocale(locale: string) {
      props.app.setPreferredLocale(locale);
      history.goBack();
    },
    onClose() {
      history.goBack();
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(SelectLanguage);
