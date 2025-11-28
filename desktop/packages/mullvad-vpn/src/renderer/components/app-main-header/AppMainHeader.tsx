import { TunnelState } from '../../../shared/daemon-rpc-types';
import { Flex, HeaderProps, Logo, LogoProps, MainHeader } from '../../lib/components';
import { useSelector } from '../../redux/store';
import { InitialFocus } from '../initial-focus';
import {
  AppMainHeaderBarAccountButton,
  AppMainHeaderDeviceInfo,
  AppMainHeaderSettingsButton,
} from './components';
import { AppMainHeaderRouterButton } from './components/AppMainHeaderRouterButton';

export interface MainHeaderProps extends Omit<HeaderProps, 'variant' | 'size'> {
  variant?: HeaderProps['variant'] | 'basedOnConnectionStatus';
  size?: HeaderProps['size'] | 'basedOnLoginStatus';
  logoVariant?: LogoProps['variant'] | 'none';
  children?: React.ReactNode;
}

const AppMainHeader = ({
  logoVariant = 'both',
  variant: variantProp,
  size: sizeProp,
  children,
  ...props
}: MainHeaderProps) => {
  const connectionStatus = useSelector((state) => state.connection.status);

  const variant =
    variantProp === 'basedOnConnectionStatus'
      ? getVariantByTunnelState(connectionStatus)
      : variantProp;

  const loggedIn = useSelector((state) => state.account.status.type === 'ok');
  const size = sizeProp === 'basedOnLoginStatus' ? (loggedIn ? '2' : '1') : sizeProp;

  return (
    <MainHeader variant={variant} size={size} {...props}>
      <Flex $justifyContent="space-between">
        <InitialFocus>
          {logoVariant !== 'none' ? <Logo variant={logoVariant} /> : <div />}
        </InitialFocus>
        <Flex $gap="tiny" $alignItems="center">
          {children}
        </Flex>
      </Flex>
      {size == '2' && (
        <Flex $alignItems="flex-end">
          <AppMainHeaderDeviceInfo />
        </Flex>
      )}
    </MainHeader>
  );
};

const AppMainHeaderNamespace = Object.assign(AppMainHeader, {
  AccountButton: AppMainHeaderBarAccountButton,
  SettingsButton: AppMainHeaderSettingsButton,
  RouterButton: AppMainHeaderRouterButton,
});

export { AppMainHeaderNamespace as AppMainHeader };

const getVariantByTunnelState = (tunnelState: TunnelState): HeaderProps['variant'] => {
  switch (tunnelState.state) {
    case 'disconnected':
      return 'error';
    case 'connecting':
    case 'connected':
      return 'success';
    case 'error':
      return !tunnelState.details.blockingError ? 'success' : 'error';
    case 'disconnecting':
      switch (tunnelState.details) {
        case 'block':
        case 'reconnect':
          return 'success';
        case 'nothing':
          return 'error';
      }
  }
};
