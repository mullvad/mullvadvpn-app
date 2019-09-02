import { goBack, push } from 'connected-react-router';
import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { links } from '../../config.json';
import WireguardKeys from '../components/WireguardKeys';
import { IWgKey } from '../redux/settings/reducers';
import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState, props: ISharedRouteProps) => ({
  keyState: state.settings.wireguardKeyState,
  isOffline: state.connection.isBlocked,
  locale: props.locale,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ push, goBack }, dispatch);
  return {
    onClose: () => history.goBack(),
    onGenerateKey: () => props.app.generateWireguardKey(),
    onReplaceKey: (oldKey: IWgKey) => props.app.replaceWireguardKey(oldKey),
    onVerifyKey: (publicKey: IWgKey) => props.app.verifyWireguardKey(publicKey),
    onVisitWebsiteKey: () => shell.openExternal(links.manageKeys),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(WireguardKeys);
