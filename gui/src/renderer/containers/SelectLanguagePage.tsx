import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import SelectLanguage from '../components/SelectLanguage';
import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  preferredLocale: state.settings.guiSettings.preferredLocale,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: RouteComponentProps & IAppContext) => {
  return {
    preferredLocalesList: props.app.getPreferredLocaleList(),
    async setPreferredLocale(locale: string) {
      await props.app.setPreferredLocale(locale);
      props.history.goBack();
    },
    onClose() {
      props.history.goBack();
    },
  };
};

export default withAppContext(
  withRouter(connect(mapStateToProps, mapDispatchToProps)(SelectLanguage)),
);
