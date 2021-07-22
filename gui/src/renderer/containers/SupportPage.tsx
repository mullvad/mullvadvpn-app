import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Support from '../components/Support';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { IReduxState, ReduxDispatch } from '../redux/store';
import supportActions from '../redux/support/actions';

const mapStateToProps = (state: IReduxState) => ({
  defaultEmail: state.support.email,
  defaultMessage: state.support.message,
  accountHistory: state.account.accountHistory,
  isOffline: state.connection.isBlocked,
  outdatedVersion: state.version.suggestedUpgrade ? true : false,
  suggestedIsBeta: state.version.suggestedIsBeta ?? false,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext & IHistoryProps) => {
  const { saveReportForm, clearReportForm } = bindActionCreators(supportActions, dispatch);

  return {
    onClose() {
      props.history.pop();
    },
    viewLog(id: string) {
      void props.app.viewLog(id);
    },
    saveReportForm,
    clearReportForm,
    collectProblemReport: props.app.collectProblemReport,
    sendProblemReport: props.app.sendProblemReport,
    onExternalLink: (url: string) => props.app.openUrl(url),
  };
};

export default withAppContext(withHistory(connect(mapStateToProps, mapDispatchToProps)(Support)));
