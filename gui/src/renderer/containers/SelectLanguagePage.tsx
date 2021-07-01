import { connect } from 'react-redux';
import SelectLanguage from '../components/SelectLanguage';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  preferredLocale: state.settings.guiSettings.preferredLocale,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    preferredLocalesList: props.app.getPreferredLocaleList(),
    async setPreferredLocale(locale: string) {
      await props.app.setPreferredLocale(locale);
      props.history.pop();
    },
    onClose() {
      props.history.pop();
    },
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(SelectLanguage)),
);
