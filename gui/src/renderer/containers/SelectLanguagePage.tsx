import { goBack } from 'connected-react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLanguage from '../components/SelectLanguage';
import withAppContext, { IAppReduxContext } from '../redux/context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  preferredLocale: state.settings.guiSettings.preferredLocale,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppReduxContext) => {
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

export default withAppContext(
  connect(
    mapStateToProps,
    mapDispatchToProps,
  )(SelectLanguage),
);
