import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import { links } from '../../config.json';
import WireguardKeys from '../components/WireguardKeys';
import withAppContext, { IAppContext } from '../context';
import { IWgKey } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  keyState: state.settings.wireguardKeyState,
  isOffline: state.connection.isBlocked,
  locale: state.userInterface.locale,
  tunnelState: state.connection.status,
  windowFocused: state.userInterface.windowFocused,
});
const mapDispatchToProps = (_dispatch: ReduxDispatch, props: RouteComponentProps & IAppContext) => {
  return {
    onClose: () => props.history.goBack(),
    onGenerateKey: () => props.app.generateWireguardKey(),
    onReplaceKey: (oldKey: IWgKey) => props.app.replaceWireguardKey(oldKey),
    onVerifyKey: (publicKey: IWgKey) => props.app.verifyWireguardKey(publicKey),
    onVisitWebsiteKey: () => props.app.openLinkWithAuth(links.manageKeys),
  };
};

export default withAppContext(
  withRouter(connect(mapStateToProps, mapDispatchToProps)(WireguardKeys)),
);
