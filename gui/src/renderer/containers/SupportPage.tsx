import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import { bindActionCreators } from 'redux';
import consumePromise from '../../shared/promise';
import Support from '../components/Support';
import withAppContext, { IAppContext } from '../context';
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

const mapDispatchToProps = (dispatch: ReduxDispatch, props: IAppContext & RouteComponentProps) => {
  const { saveReportForm, clearReportForm } = bindActionCreators(supportActions, dispatch);

  return {
    onClose() {
      props.history.goBack();
    },
    viewLog(id: string) {
      consumePromise(props.app.viewLog(id));
    },
    saveReportForm,
    clearReportForm,
    collectProblemReport: props.app.collectProblemReport,
    sendProblemReport: props.app.sendProblemReport,
    onExternalLink: (url: string) => props.app.openUrl(url),
  };
};

export default withAppContext(withRouter(connect(mapStateToProps, mapDispatchToProps)(Support)));
