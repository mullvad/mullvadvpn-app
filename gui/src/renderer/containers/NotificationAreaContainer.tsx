import { shell } from 'electron';
import { connect } from 'react-redux';
import NotificationArea from '../components/NotificationArea';
import { links } from '../../config.json';
import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';
import AccountExpiry from '../lib/account-expiry';

const mapStateToProps = (state: IReduxState) => ({
  accountExpiry: state.account.expiry
    ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
    : undefined,
  tunnelState: state.connection.status,
  version: state.version,
  blockWhenDisconnected: state.settings.blockWhenDisconnected,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, _props: ISharedRouteProps) => {
  return {
    onOpenDownloadLink() {
      shell.openExternal(links.download);
    },
    onOpenBuyMoreLink() {
      shell.openExternal(links.purchase);
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(NotificationArea);
