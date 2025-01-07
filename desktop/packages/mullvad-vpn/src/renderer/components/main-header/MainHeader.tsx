import { TunnelState } from '../../../shared/daemon-rpc-types';
import { Flex, Header, HeaderProps, Logo, LogoProps } from '../../lib/components';
import { Spacings } from '../../lib/foundations';
import { useSelector } from '../../redux/store';
import { FocusFallback } from '../Focus';
import {
  MainHeaderBarAccountButton,
  MainHeaderDeviceInfo,
  MainHeaderSettingsButton,
} from './components';

export interface MainHeaderProps extends Omit<HeaderProps, 'variant' | 'size'> {
  variant?: HeaderProps['variant'] | 'basedOnConnectionStatus';
  size?: HeaderProps['size'] | 'basedOnLoginStatus';
  logoVariant?: LogoProps['variant'] | 'none';
  children?: React.ReactNode;
}

const MainHeader = ({
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
    <Header variant={variant} size={size} {...props}>
      <Header.MainRow>
        <FocusFallback>
          {logoVariant !== 'none' ? <Logo variant={logoVariant} /> : <div />}
        </FocusFallback>
        <Flex $gap={Spacings.spacing5} $alignItems="center">
          {children}
        </Flex>
      </Header.MainRow>
      {size == '2' && (
        <Header.SubRow>
          <MainHeaderDeviceInfo />
        </Header.SubRow>
      )}
    </Header>
  );
};

const MainHeaderNamespace = Object.assign(MainHeader, {
  AccountButton: MainHeaderBarAccountButton,
  SettingsButton: MainHeaderSettingsButton,
});

export { MainHeaderNamespace as MainHeader };

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
