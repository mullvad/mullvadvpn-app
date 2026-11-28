import { TunnelState } from '../../../shared/daemon-rpc-types';
import { Flex, HeaderProps, Logo, LogoProps, MainHeader } from '../../lib/components';
import { useSelector } from '../../redux/store';
import { InitialFocus } from '../initial-focus';
import { AppMainHeaderBarConfigButton, AppMainHeaderSettingsButton } from './components';

export interface MainHeaderProps extends Omit<HeaderProps, 'variant' | 'size'> {
  variant?: HeaderProps['variant'] | 'basedOnConnectionStatus';
  size?: HeaderProps['size'];
  logoVariant?: LogoProps['variant'] | 'none';
  children?: React.ReactNode;
}

const AppMainHeader = ({
  logoVariant = 'both',
  variant: variantProp,
  size,
  children,
  ...props
}: MainHeaderProps) => {
  const connectionStatus = useSelector((state) => state.connection.status);

  const variant =
    variantProp === 'basedOnConnectionStatus'
      ? getVariantByTunnelState(connectionStatus)
      : variantProp;

  return (
    <MainHeader variant={variant} size={size} {...props}>
      <Flex justifyContent="space-between">
        <InitialFocus>
          {logoVariant !== 'none' ? <Logo variant={logoVariant} /> : <div />}
        </InitialFocus>
        <Flex gap="medium" alignItems="center">
          {children}
        </Flex>
      </Flex>
    </MainHeader>
  );
};

const AppMainHeaderNamespace = Object.assign(AppMainHeader, {
  ConfigButton: AppMainHeaderBarConfigButton,
  SettingsButton: AppMainHeaderSettingsButton,
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
