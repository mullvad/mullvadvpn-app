import { connect } from 'react-redux';

import log from '../../shared/logging';
import ExpiredAccountErrorView from '../components/ExpiredAccountErrorView';
import withAppContext, { IAppContext } from '../context';
import { IHistoryProps, withHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  accountToken: state.account.accountToken,
  loginState: state.account.status,
  tunnelState: state.connection.status,
  isBlocked: state.connection.isBlocked,
  blockWhenDisconnected: state.settings.blockWhenDisconnected,
});
const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IHistoryProps & IAppContext) => {
  return {
    onExternalLinkWithAuth: (url: string) => props.app.openLinkWithAuth(url),
    onDisconnect: async () => {
      try {
        await props.app.disconnectTunnel();
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to disconnect the tunnel: ${error.message}`);
      }
    },
    setBlockWhenDisconnected: async (blockWhenDisconnected: boolean) => {
      try {
        await props.app.setBlockWhenDisconnected(blockWhenDisconnected);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update block when disconnected', error.message);
      }
    },
    navigateToRedeemVoucher: () => {
      props.history.push(RoutePath.redeemVoucher);
    },
  };
};

export default withAppContext(
  withHistory(connect(mapStateToProps, mapDispatchToProps)(ExpiredAccountErrorView)),
);
