// @flow
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Support from '../components/Support';
import { openItem } from '../lib/platform';
import { collectProblemReport, sendProblemReport } from '../lib/problem-report';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => state;

const mapDispatchToProps = (dispatch: ReduxDispatch, _props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);

  return {
    onClose: () => pushHistory('/settings'),

    onCollectLog: (toRedact) => {
      return collectProblemReport(toRedact);
    },

    onViewLog: (path) => openItem(path),

    onSend: (email, message, savedReport) => {
      return sendProblemReport(email, message, savedReport);
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Support);
